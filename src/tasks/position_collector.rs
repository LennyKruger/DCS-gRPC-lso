//! Priority acquisition of paired carrier/aircraft transforms.
//!
//! The buffered implementation consumes source-captured snapshots in sequence.
//! The unary implementation is retained as an explicit diagnostic rollback.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use stubs::recovery::v0::recovery_service_client::RecoveryServiceClient;
use stubs::recovery::v0::{
    ReadRecoveryTelemetryRequest, RecoveryTelemetryLifecycleStatus, RecoveryTelemetryLossReason,
    RecoveryTelemetrySnapshot, RecoveryTransform, StartRecoveryTelemetryRequest,
    StopRecoveryTelemetryRequest, UnitObservationStatus,
};

use crate::client::{request_with_deadline, GrpcChannel, GrpcResult, UnitClient};
use crate::metrics::RpcKind;
use crate::telemetry::{
    InvalidSourceObservation, ScoringSegmentAttribution, SourceObservationEntity, TelemetryAligner,
    TelemetrySample,
};
use crate::track::{OnlineMetricStats, PositionCollectionMetrics};
use crate::transform::Transform;

use super::PositionSource;

static HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct BufferedCollectionDiagnostics {
    pub source_epoch: String,
    pub last_sequence: u64,
    pub last_capture_tick: u64,
    pub read_batches: u64,
    pub snapshots_received: u64,
    pub invalid_snapshots: u64,
    pub lost_snapshots: u64,
    /// Reader-observed missing sequence count. Unlike ring churn counters, non-zero means the
    /// client actually requested data that was no longer available.
    pub observed_reader_sequence_losses: u64,
    pub reader_sequence_contiguous: bool,
    pub overflow_count: u64,
    pub missed_capture_intervals: u64,
    pub retention_expiration_count: u64,
    pub capacity_overflow_count: u64,
    /// Explicit semantic alias: source-side capacity evictions/churn, not snapshots proven lost
    /// by this reader. Kept alongside the legacy field for schema-v3 compatibility.
    pub source_ring_capacity_evictions: u64,
    pub source_ring_retention_evictions: u64,
    pub high_water_mark: u32,
    pub configured_period_ms: f64,
    pub retention_seconds: f64,
    pub capacity: u32,
}

#[derive(Debug, Default)]
pub struct PositionBatch {
    pub samples: Vec<TelemetrySample>,
    pub lost_snapshots: u64,
    pub invalid_snapshots: u64,
    pub invalid_observations: Vec<InvalidSourceObservation>,
}

enum PositionCollectorKind {
    Unary {
        carrier: UnitClient,
        plane: UnitClient,
        aligner: Box<TelemetryAligner>,
    },
    Buffered {
        svc: RecoveryServiceClient<GrpcChannel>,
        handle: String,
        source_epoch: String,
        after_sequence: u64,
        previous_capture_time: Option<f64>,
        diagnostics: BufferedCollectionDiagnostics,
        stopped: bool,
    },
}

pub struct PositionCollector {
    kind: PositionCollectorKind,
    poll_latency_stats: OnlineMetricStats,
    errors: u32,
    timeouts: u32,
}

impl PositionCollector {
    #[allow(clippy::too_many_arguments)]
    pub async fn start(
        channel: GrpcChannel,
        source: PositionSource,
        session_id: i64,
        generation: u64,
        carrier_id: u32,
        carrier_name: &str,
        plane_id: u32,
        plane_name: &str,
    ) -> GrpcResult<Self> {
        let kind = match source {
            PositionSource::Unary => PositionCollectorKind::Unary {
                carrier: UnitClient::new(channel.clone()),
                plane: UnitClient::new(channel),
                aligner: Box::new(TelemetryAligner::new()),
            },
            PositionSource::Buffered => {
                let nonce = HANDLE_COUNTER.fetch_add(1, Ordering::Relaxed);
                let handle =
                    format!("s{session_id}-g{generation}-p{plane_id}-c{carrier_id}-n{nonce}");
                let mut svc = RecoveryServiceClient::new(channel);
                let started = svc
                    .start_recovery_telemetry(request_with_deadline(
                        StartRecoveryTelemetryRequest {
                            recovery_handle: handle.clone(),
                            aircraft_name: plane_name.to_string(),
                            aircraft_id: plane_id,
                            carrier_name: carrier_name.to_string(),
                            carrier_id,
                        },
                    ))
                    .await
                    .map_err(Box::new)?
                    .into_inner();
                if started.source_epoch.is_empty() || started.recovery_handle != handle {
                    return Err(Box::new(tonic::Status::data_loss(
                        "invalid StartRecoveryTelemetry identity response",
                    )));
                }
                let diagnostics = BufferedCollectionDiagnostics {
                    source_epoch: started.source_epoch.clone(),
                    configured_period_ms: started.configured_period * 1_000.0,
                    retention_seconds: started.retention_seconds,
                    capacity: started.capacity,
                    reader_sequence_contiguous: true,
                    ..BufferedCollectionDiagnostics::default()
                };
                tracing::info!(
                    recovery_handle = %handle,
                    source_epoch = %started.source_epoch,
                    configured_period_ms = diagnostics.configured_period_ms,
                    capacity = diagnostics.capacity,
                    retention_seconds = diagnostics.retention_seconds,
                    already_active = started.already_active,
                    "source-buffered recovery telemetry started"
                );
                PositionCollectorKind::Buffered {
                    svc,
                    handle,
                    source_epoch: started.source_epoch,
                    after_sequence: 0,
                    previous_capture_time: None,
                    diagnostics,
                    stopped: false,
                }
            }
        };
        Ok(Self {
            kind,
            poll_latency_stats: OnlineMetricStats::default(),
            errors: 0,
            timeouts: 0,
        })
    }

    pub async fn poll(
        &mut self,
        carrier_name: &str,
        plane_name: &str,
    ) -> GrpcResult<PositionBatch> {
        let started = std::time::Instant::now();
        let result = match &mut self.kind {
            PositionCollectorKind::Unary {
                carrier,
                plane,
                aligner,
            } => {
                let pair = futures_util::future::try_join(
                    carrier.get_observed_transform_for(carrier_name, RpcKind::TransformCarrier),
                    plane.get_observed_transform_for(plane_name, RpcKind::TransformPlane),
                )
                .await;
                pair.map(|(carrier, plane)| PositionBatch {
                    samples: vec![aligner.align(carrier, plane)],
                    ..PositionBatch::default()
                })
            }
            PositionCollectorKind::Buffered {
                svc,
                handle,
                source_epoch,
                after_sequence,
                previous_capture_time,
                diagnostics,
                ..
            } => {
                read_buffered(
                    svc,
                    handle,
                    source_epoch,
                    after_sequence,
                    previous_capture_time,
                    diagnostics,
                )
                .await
            }
        };
        self.poll_latency_stats
            .observe(started.elapsed().as_secs_f64() * 1_000.0);
        if let Err(error) = &result {
            self.errors = self.errors.saturating_add(1);
            if error.code() == tonic::Code::DeadlineExceeded {
                self.timeouts = self.timeouts.saturating_add(1);
            }
        }
        result
    }

    pub fn reset(&mut self) {
        if let PositionCollectorKind::Unary { aligner, .. } = &mut self.kind {
            aligner.reset();
        }
    }

    pub async fn stop(&mut self) -> GrpcResult<()> {
        let PositionCollectorKind::Buffered {
            svc,
            handle,
            source_epoch,
            stopped,
            ..
        } = &mut self.kind
        else {
            return Ok(());
        };
        let response = svc
            .stop_recovery_telemetry(request_with_deadline(StopRecoveryTelemetryRequest {
                recovery_handle: handle.clone(),
                expected_source_epoch: source_epoch.clone(),
            }))
            .await
            .map_err(Box::new)?
            .into_inner();
        let lifecycle = RecoveryTelemetryLifecycleStatus::try_from(response.lifecycle_status)
            .unwrap_or(RecoveryTelemetryLifecycleStatus::Unspecified);
        if !matches!(
            lifecycle,
            RecoveryTelemetryLifecycleStatus::Stopped
                | RecoveryTelemetryLifecycleStatus::Unknown
                | RecoveryTelemetryLifecycleStatus::Expired
        ) {
            return Err(Box::new(tonic::Status::failed_precondition(format!(
                "unexpected StopRecoveryTelemetry lifecycle: {lifecycle:?}"
            ))));
        }
        tracing::info!(recovery_handle = %handle, ?lifecycle, "source-buffered recovery telemetry stopped");
        *stopped = true;
        Ok(())
    }

    pub fn acquisition_source(&self) -> &'static str {
        match self.kind {
            PositionCollectorKind::Unary { .. } => PositionSource::Unary.acquisition_source(),
            PositionCollectorKind::Buffered { .. } => PositionSource::Buffered.acquisition_source(),
        }
    }

    pub fn is_buffered(&self) -> bool {
        matches!(self.kind, PositionCollectorKind::Buffered { .. })
    }

    pub fn buffered_diagnostics(&self) -> Option<&BufferedCollectionDiagnostics> {
        match &self.kind {
            PositionCollectorKind::Buffered { diagnostics, .. } => Some(diagnostics),
            PositionCollectorKind::Unary { .. } => None,
        }
    }

    pub fn metrics(&self) -> PositionCollectionMetrics {
        PositionCollectionMetrics {
            polls: self.poll_latency_stats.count().min(u64::from(u32::MAX)) as u32,
            errors: self.errors,
            timeouts: self.timeouts,
            mean_latency_ms: self.poll_latency_stats.mean(),
            p50_latency_ms: self.poll_latency_stats.percentile(0.50),
            p95_latency_ms: self.poll_latency_stats.percentile(0.95),
            p99_latency_ms: self.poll_latency_stats.percentile(0.99),
            max_latency_ms: self.poll_latency_stats.max(),
        }
    }
}

impl Drop for PositionCollector {
    fn drop(&mut self) {
        let PositionCollectorKind::Buffered {
            svc,
            handle,
            source_epoch,
            stopped,
            ..
        } = &self.kind
        else {
            return;
        };
        if *stopped {
            return;
        }
        let mut svc = svc.clone();
        let handle = handle.clone();
        let source_epoch = source_epoch.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                let result = svc
                    .stop_recovery_telemetry(request_with_deadline(
                        StopRecoveryTelemetryRequest {
                            recovery_handle: handle.clone(),
                            expected_source_epoch: source_epoch,
                        },
                    ))
                    .await;
                if let Err(status) = result {
                    tracing::warn!(?status, recovery_handle = %handle, "best-effort telemetry cleanup failed");
                }
            });
        }
    }
}

async fn read_buffered(
    svc: &mut RecoveryServiceClient<GrpcChannel>,
    handle: &str,
    source_epoch: &str,
    after_sequence: &mut u64,
    previous_capture_time: &mut Option<f64>,
    diagnostics: &mut BufferedCollectionDiagnostics,
) -> GrpcResult<PositionBatch> {
    let response = svc
        .read_recovery_telemetry(request_with_deadline(ReadRecoveryTelemetryRequest {
            recovery_handle: handle.to_string(),
            expected_source_epoch: source_epoch.to_string(),
            after_sequence: *after_sequence,
            limit: 100,
        }))
        .await
        .map_err(Box::new)?
        .into_inner();

    let lifecycle = RecoveryTelemetryLifecycleStatus::try_from(response.lifecycle_status)
        .unwrap_or(RecoveryTelemetryLifecycleStatus::Unspecified);
    if lifecycle != RecoveryTelemetryLifecycleStatus::Active {
        return Err(Box::new(tonic::Status::failed_precondition(format!(
            "ReadRecoveryTelemetry lifecycle is {lifecycle:?}"
        ))));
    }
    if response.source_epoch != source_epoch || response.recovery_handle != handle {
        return Err(Box::new(tonic::Status::data_loss(
            "ReadRecoveryTelemetry returned a different epoch or handle",
        )));
    }

    let loss_reason = RecoveryTelemetryLossReason::try_from(response.loss_reason)
        .unwrap_or(RecoveryTelemetryLossReason::Unspecified);
    let lost_snapshots = reported_loss_count(
        loss_reason,
        response.oldest_available_sequence,
        *after_sequence,
    );

    let mut samples = Vec::with_capacity(response.snapshots.len());
    let mut invalid_snapshots = 0_u64;
    let mut invalid_observations = Vec::new();
    let received_unix_ms = unix_time_ms();
    let mut previous_sequence = if lost_snapshots == 0 {
        *after_sequence
    } else {
        response.oldest_available_sequence.saturating_sub(1)
    };
    for snapshot in response.snapshots {
        previous_sequence = advance_sequence(previous_sequence, snapshot.sequence)?;
        diagnostics.last_capture_tick = snapshot.capture_tick;
        let source_invalid =
            invalid_observations_in_snapshot(&snapshot, response.read_time, received_unix_ms)?;
        if !source_invalid.is_empty() {
            invalid_snapshots = invalid_snapshots.saturating_add(1);
            invalid_observations.extend(source_invalid);
            continue;
        }
        match snapshot_to_sample(snapshot, response.read_time, *previous_capture_time) {
            Ok(sample) => {
                *previous_capture_time = Some(sample.carrier.time.max(sample.plane.time));
                samples.push(sample);
            }
            Err(status) if status.code() == tonic::Code::FailedPrecondition => {
                invalid_snapshots = invalid_snapshots.saturating_add(1);
            }
            Err(status) => return Err(status),
        }
    }
    if response.next_after_sequence < *after_sequence
        || response.next_after_sequence != previous_sequence
    {
        return Err(Box::new(tonic::Status::data_loss(
            "invalid next_after_sequence in recovery telemetry response",
        )));
    }
    *after_sequence = response.next_after_sequence;

    diagnostics.last_sequence = *after_sequence;
    diagnostics.read_batches = diagnostics.read_batches.saturating_add(1);
    diagnostics.snapshots_received = diagnostics
        .snapshots_received
        .saturating_add(samples.len() as u64 + invalid_snapshots);
    diagnostics.invalid_snapshots = diagnostics
        .invalid_snapshots
        .saturating_add(invalid_snapshots);
    diagnostics.lost_snapshots = diagnostics.lost_snapshots.saturating_add(lost_snapshots);
    diagnostics.observed_reader_sequence_losses = diagnostics.lost_snapshots;
    diagnostics.reader_sequence_contiguous &= lost_snapshots == 0;
    diagnostics.overflow_count = response.overflow_count;
    diagnostics.configured_period_ms = response.configured_period * 1_000.0;
    diagnostics.retention_seconds = response.retention_seconds;
    diagnostics.capacity = response.capacity;
    if let Some(source) = response.diagnostics {
        diagnostics.missed_capture_intervals = source.missed_capture_intervals;
        diagnostics.retention_expiration_count = source.retention_expiration_count;
        diagnostics.capacity_overflow_count = source.capacity_overflow_count;
        diagnostics.source_ring_retention_evictions = source.retention_expiration_count;
        diagnostics.source_ring_capacity_evictions = source.capacity_overflow_count;
        diagnostics.high_water_mark = source.high_water_mark;
    }

    Ok(PositionBatch {
        samples,
        lost_snapshots,
        invalid_snapshots,
        invalid_observations,
    })
}

fn invalid_observations_in_snapshot(
    snapshot: &RecoveryTelemetrySnapshot,
    read_time: f64,
    received_unix_ms: u64,
) -> GrpcResult<Vec<InvalidSourceObservation>> {
    let capture_time_dcs = snapshot
        .capture_time
        .is_finite()
        .then_some(snapshot.capture_time);
    let source_read_time_dcs = read_time.is_finite().then_some(read_time);
    let mut invalid = Vec::new();
    for (entity, observation) in [
        (
            SourceObservationEntity::Aircraft,
            snapshot.aircraft.as_ref(),
        ),
        (SourceObservationEntity::Carrier, snapshot.carrier.as_ref()),
    ] {
        let observation = observation.ok_or_else(|| {
            Box::new(tonic::Status::data_loss(format!(
                "missing {entity:?} observation in telemetry snapshot"
            )))
        })?;
        let status_code = observation.status;
        let status = unit_observation_status_name(status_code);
        if status_code != UnitObservationStatus::Valid as i32 {
            invalid.push(InvalidSourceObservation {
                sequence: snapshot.sequence,
                capture_tick: snapshot.capture_tick,
                capture_time_dcs,
                entity,
                status_code,
                status: status.clone(),
                reason: status,
                source_read_time_dcs,
                received_unix_ms,
                attribution: ScoringSegmentAttribution::IndeterminateMissingSourceTime,
                affects_scoring: false,
            });
        }
    }
    Ok(invalid)
}

fn unit_observation_status_name(status_code: i32) -> String {
    match UnitObservationStatus::try_from(status_code) {
        Ok(UnitObservationStatus::Unspecified) => "unspecified".to_string(),
        Ok(UnitObservationStatus::Valid) => "valid".to_string(),
        Ok(UnitObservationStatus::NotFound) => "not_found".to_string(),
        Ok(UnitObservationStatus::IdMismatch) => "id_mismatch".to_string(),
        Ok(UnitObservationStatus::ReadError) => "read_error".to_string(),
        Ok(UnitObservationStatus::InvalidData) => "invalid_data".to_string(),
        Err(_) => format!("unknown_status_{status_code}"),
    }
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn reported_loss_count(
    reason: RecoveryTelemetryLossReason,
    oldest_available_sequence: u64,
    after_sequence: u64,
) -> u64 {
    if matches!(
        reason,
        RecoveryTelemetryLossReason::RetentionExpired
            | RecoveryTelemetryLossReason::CapacityOverflow
            | RecoveryTelemetryLossReason::Mixed
    ) {
        oldest_available_sequence
            .saturating_sub(after_sequence.saturating_add(1))
            .max(1)
    } else {
        0
    }
}

fn advance_sequence(previous: u64, received: u64) -> GrpcResult<u64> {
    let expected = previous.saturating_add(1);
    if received != expected {
        return Err(Box::new(tonic::Status::data_loss(format!(
            "non-contiguous recovery telemetry sequence: expected {expected}, received {received}"
        ))));
    }
    Ok(received)
}

fn snapshot_to_sample(
    snapshot: RecoveryTelemetrySnapshot,
    read_time: f64,
    previous_capture_time: Option<f64>,
) -> GrpcResult<TelemetrySample> {
    if !snapshot.capture_time.is_finite() || !read_time.is_finite() {
        return Err(Box::new(tonic::Status::data_loss(
            "non-finite recovery telemetry timestamp",
        )));
    }
    let aircraft = valid_transform(snapshot.aircraft, "aircraft")?;
    let carrier = valid_transform(snapshot.carrier, "carrier")?;
    let capture_time = snapshot.capture_time;
    let source_age_ms = ((read_time - capture_time).max(0.0)) * 1_000.0;
    Ok(TelemetrySample::from_source_pair(
        transform_at(carrier, capture_time)?,
        transform_at(aircraft, capture_time)?,
        previous_capture_time,
        source_age_ms,
    ))
}

fn valid_transform(
    observation: Option<stubs::recovery::v0::RecoveryUnitObservation>,
    side: &str,
) -> GrpcResult<RecoveryTransform> {
    let observation = observation.ok_or_else(|| {
        Box::new(tonic::Status::data_loss(format!(
            "missing {side} observation in telemetry snapshot"
        )))
    })?;
    let status = UnitObservationStatus::try_from(observation.status)
        .unwrap_or(UnitObservationStatus::Unspecified);
    if status != UnitObservationStatus::Valid {
        return Err(Box::new(tonic::Status::failed_precondition(format!(
            "invalid {side} observation status: {status:?}"
        ))));
    }
    observation.transform.ok_or_else(|| {
        Box::new(tonic::Status::data_loss(format!(
            "valid {side} observation has no transform"
        )))
    })
}

fn transform_at(transform: RecoveryTransform, capture_time: f64) -> GrpcResult<Transform> {
    let position = transform.position.ok_or_else(|| {
        Box::new(tonic::Status::data_loss(
            "valid recovery transform has no position",
        ))
    })?;
    let orientation = transform.orientation.ok_or_else(|| {
        Box::new(tonic::Status::data_loss(
            "valid recovery transform has no orientation",
        ))
    })?;
    let velocity = transform.velocity.ok_or_else(|| {
        Box::new(tonic::Status::data_loss(
            "valid recovery transform has no velocity",
        ))
    })?;
    let finite_vector = |vector: &Option<stubs::common::v0::Vector>| {
        vector
            .as_ref()
            .is_some_and(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
    };
    let scalars_are_finite = [
        position.lat,
        position.lon,
        position.alt,
        position.u,
        position.v,
        orientation.heading,
        orientation.yaw,
        orientation.pitch,
        orientation.roll,
        velocity.heading,
        velocity.speed,
    ]
    .into_iter()
    .all(f64::is_finite);
    if !scalars_are_finite
        || !finite_vector(&orientation.forward)
        || !finite_vector(&orientation.right)
        || !finite_vector(&orientation.up)
        || !finite_vector(&velocity.velocity)
    {
        return Err(Box::new(tonic::Status::data_loss(
            "valid recovery transform is incomplete or non-finite",
        )));
    }
    Ok((capture_time, position, orientation, velocity).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use stubs::common::v0::{Orientation, Position, Vector, Velocity};
    use stubs::recovery::v0::RecoveryUnitObservation;

    fn observation() -> RecoveryUnitObservation {
        RecoveryUnitObservation {
            status: UnitObservationStatus::Valid.into(),
            transform: Some(RecoveryTransform {
                position: Some(Position::default()),
                orientation: Some(Orientation {
                    forward: Some(Vector::default()),
                    right: Some(Vector::default()),
                    up: Some(Vector::default()),
                    ..Orientation::default()
                }),
                velocity: Some(Velocity {
                    velocity: Some(Vector::default()),
                    ..Velocity::default()
                }),
            }),
            ..RecoveryUnitObservation::default()
        }
    }

    #[test]
    fn source_snapshot_uses_one_capture_timestamp() {
        let sample = snapshot_to_sample(
            RecoveryTelemetrySnapshot {
                sequence: 1,
                capture_time: 42.5,
                aircraft: Some(observation()),
                carrier: Some(observation()),
                ..RecoveryTelemetrySnapshot::default()
            },
            42.8,
            None,
        )
        .unwrap();
        assert_eq!(sample.carrier.time, 42.5);
        assert_eq!(sample.plane.time, 42.5);
        assert!((sample.source_age_ms - 300.0).abs() < 1.0e-6);
    }

    #[test]
    fn invalid_unit_status_is_not_turned_into_a_position() {
        let status = snapshot_to_sample(
            RecoveryTelemetrySnapshot {
                sequence: 1,
                capture_time: 42.5,
                aircraft: Some(RecoveryUnitObservation {
                    status: UnitObservationStatus::IdMismatch.into(),
                    ..RecoveryUnitObservation::default()
                }),
                carrier: Some(observation()),
                ..RecoveryTelemetrySnapshot::default()
            },
            42.6,
            None,
        )
        .unwrap_err();
        assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    }

    #[test]
    fn invalid_source_observations_preserve_side_status_sequence_and_clocks() {
        let snapshot = RecoveryTelemetrySnapshot {
            sequence: 17,
            capture_tick: 91,
            capture_time: 42.5,
            aircraft: Some(RecoveryUnitObservation {
                status: UnitObservationStatus::IdMismatch.into(),
                ..RecoveryUnitObservation::default()
            }),
            carrier: Some(RecoveryUnitObservation {
                status: UnitObservationStatus::ReadError.into(),
                ..RecoveryUnitObservation::default()
            }),
            ..RecoveryTelemetrySnapshot::default()
        };
        let invalid = invalid_observations_in_snapshot(&snapshot, 47.0, 1_234).unwrap();
        assert_eq!(invalid.len(), 2);
        assert_eq!(invalid[0].entity, SourceObservationEntity::Aircraft);
        assert_eq!(invalid[0].status, "id_mismatch");
        assert_eq!(invalid[1].entity, SourceObservationEntity::Carrier);
        assert_eq!(invalid[1].status, "read_error");
        assert!(invalid.iter().all(|item| {
            item.sequence == 17
                && item.capture_tick == 91
                && item.capture_time_dcs == Some(42.5)
                && item.source_read_time_dcs == Some(47.0)
                && item.received_unix_ms == 1_234
                && item.reason == item.status
        }));
    }

    #[test]
    fn non_finite_source_time_is_explicitly_missing_not_replaced_by_read_time() {
        let snapshot = RecoveryTelemetrySnapshot {
            sequence: 3,
            capture_time: f64::NAN,
            aircraft: Some(RecoveryUnitObservation {
                status: UnitObservationStatus::NotFound.into(),
                ..RecoveryUnitObservation::default()
            }),
            carrier: Some(observation()),
            ..RecoveryTelemetrySnapshot::default()
        };
        let invalid = invalid_observations_in_snapshot(&snapshot, 50.0, 5_000).unwrap();
        assert_eq!(invalid[0].capture_time_dcs, None);
        assert_eq!(invalid[0].source_read_time_dcs, Some(50.0));
        assert_eq!(invalid[0].status, "not_found");
    }

    #[test]
    fn incomplete_valid_transform_is_not_fabricated() {
        let mut incomplete = observation();
        incomplete.transform.as_mut().unwrap().position = None;
        let status = snapshot_to_sample(
            RecoveryTelemetrySnapshot {
                sequence: 1,
                capture_time: 42.5,
                aircraft: Some(incomplete),
                carrier: Some(observation()),
                ..RecoveryTelemetrySnapshot::default()
            },
            42.6,
            None,
        )
        .unwrap_err();
        assert_eq!(status.code(), tonic::Code::DataLoss);
    }

    #[test]
    fn sequence_validation_rejects_gaps_and_duplicates() {
        assert_eq!(advance_sequence(7, 8).unwrap(), 8);
        assert_eq!(
            advance_sequence(7, 7).unwrap_err().code(),
            tonic::Code::DataLoss
        );
        assert_eq!(
            advance_sequence(7, 9).unwrap_err().code(),
            tonic::Code::DataLoss
        );
    }

    #[test]
    fn explicit_retention_or_capacity_loss_counts_missing_positions() {
        assert_eq!(
            reported_loss_count(RecoveryTelemetryLossReason::None, 10, 2),
            0
        );
        assert_eq!(
            reported_loss_count(RecoveryTelemetryLossReason::RetentionExpired, 10, 2),
            7
        );
        assert_eq!(
            reported_loss_count(RecoveryTelemetryLossReason::CapacityOverflow, 10, 8),
            1
        );
    }
}
