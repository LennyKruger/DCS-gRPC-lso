# LSO Changelog

This file records user-visible changes. The crate version remains `0.2.0`; changes after the
`0.2.0` tag are therefore listed under Unreleased.

## Unreleased

### Added

- `groove_time_secs`: the recovery report now serializes the groove duration (previously computed
  but visible only in the Discord embed), so `_OK_` eligibility can be checked without Discord
  configured.
- `wind_reference_probes`: diagnostic-only field surfacing the two raw `GetWind` responses (each
  altitude/heading/speed) behind `wind_reference_established`, plus a DEBUG-level log of each probe
  individually (and of the separate report-time `GetWind` query, previously only logged on failure).
  Added to investigate a confirmed live anomaly (two reports out of eight reading `180deg/0.0 m/s`
  against a consistent `95deg/0.99-1.42 m/s` on the other six, same ship/mission/timeframe) — no
  change to the AoA correction logic itself; see `tasking-roadmap.md` for what this is meant to
  determine on the next live test.
- Automatic `_OK_` grade (`is_amplitude_perfect`/`grade_from_gates`, `project-derived-v4`): a pass
  already `Ok` by every existing rule is upgraded to `_OK_` (5.0 points) when every gate and every
  continuous sample stays inside a tighter GS/lineup band (`PROJECT-DERIVED`, borrowed from the
  MOOSE `Ops.Airboss` mod) and `groove_time_secs` falls within the NATOPS-documented 15-18 s groove
  window (NAVAIR 00-80T-105 §6.2.4.3). Independent of the wire number caught; never available to a
  touch-and-go.
- Hook-position calibration (touch-and-go vs bolter) extended from the F/A-18C to the VNAO T-45
  (draw argument index 25, shared with the F/A-18C) and the F-14A/B/B(U) (index 1305), via new
  `AirplaneInfo::hook_draw_argument`. Only the F/A-18C's `<=0.2`=up/`>=0.8`=down polarity is
  empirically confirmed; T-45/F-14 reuse it as an unverified assumption pending live confirmation.
- A dedicated sink-rate/bank-angle Cut (`dangerous_sink_rate_or_bank`, `src/grading.rs`): a
  sustained (>=3 consecutive samples) sink rate >=8.0 m/s or bank >=30 degrees inside the
  quarter-NM grades the pass `C`. `PROJECT-DERIVED`; NATOPS documents sink rate and bank as
  waveoff-judgment factors but codifies no numeric threshold for either.
- CATOBAR groove-entry refinement (`is_rolled_out`, `src/track.rs`): in addition to the existing
  distance/altitude/lineup box, `entered_groove` now also requires near-level bank
  (`GROOVE_ROLLOUT_MAX_BANK_DEG = 15°`) and a ground track already aligned with the groove axis
  (`GROOVE_ROLLOUT_MAX_TRACK_ANGLE_DEG = 15°`), confirmed over 2 consecutive samples — an observable
  proxy for the NATOPS Case I "roll wings level on centerline with a centered ball" event that the
  box alone could not distinguish from a transient pass through it during the final turn. CATOBAR
  only; V/STOL keeps the box alone.
- A persistence guard on continuous-trajectory amplitude (`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES =
  2`): an isolated frame above the slight/significant threshold no longer counts alone. Never
  applied to the Cut threshold or the late-approach weighting, which stay sensitive to a single
  sample. An overcorrection/oscillation check (`OSCILLATION_MIN_REVERSALS`/
  `OSCILLATION_MIN_SWING_DEG`) also caps a pass at `(OK)` when GS/lineup shows repeated direction
  reversals over the same 4-second window used by the correction-trend check, even when the net
  deviation is near zero.
- `sink_rate_mps`/`bank_deg` in `trajectory_deviations`, and `roll_deg` in `datums`: contextual
  telemetry, scored only through the dedicated Cut above.
- `trajectory_deviations`: a continuous GS/lineup series computed from groove entry to touchdown
  (additive JSON field alongside `gate_deviations`), and used as a second, continuous source of
  amplitude for `PassGrade` next to the three point-in-time gates (`PROJECT-DERIVED`; see
  `AGENTS.md`).
- A correction-trend check on that same trajectory: a pass whose GS/lineup deviation is still
  measurably worsening in the final 4 seconds before touchdown is capped at `(OK)` instead of
  `Ok`, matching NATOPS' own OK ("reasonable deviations with good corrections") vs (OK) ("fair —
  reasonable deviations") distinction. Never used to raise a grade amplitude placed lower, and
  never checked once a pass is already below `Ok` (`PROJECT-DERIVED`; see
  `AGENTS.md`, "Gates, outcomes et câble").
- A late-approach severity check: a moderate GS/lineup deviation (above `LATE_WINDOW_GS_DEG`/
  `LATE_WINDOW_LU_DEG`, between the general slight/significant thresholds) found inside the last
  150 m before the ramp caps an otherwise-`Ok`/`(OK)` pass at `--` instead, since there is no
  distance left to correct it there. The identical deviation earlier in the approach is graded
  normally. Never raises a grade, never affects an already-`NoGrade`/`Cut` result, never touches
  Cut itself (`PROJECT-DERIVED`; see `AGENTS.md`, "Gates, outcomes et câble").
- `wind_heading_deg`/`wind_speed_mps`: contextual wind at the carrier's position, in the JSON
  report for every recovery (previously queried only for the Discord embed, and not persisted).
  Never affects `pass_grade`/`grade_points`; absent in `--positions-only`.
- `aoa` in `datums`/`pattern_datums` is now wind-corrected once a wind reference is established
  (two `AtmosphereService.GetWind` calls at groove entry, interpolated by altitude for the rest of
  the recovery — DCS wind is deterministic and altitude-dependent, not time-varying) instead of
  the raw nose-vs-ground-velocity angle, which was systematically biased by wind (always present
  during carrier ops) and mixed sideslip/crab into a single always-positive value. New additive
  `wind_reference_established` field records whether the correction applied; falls back to the raw
  approximation, never a fabricated value, when it did not (`PROJECT-DERIVED`; see
  `AGENTS.md`, "Gates, outcomes et câble"). Still chart/report context only, never scored.
- `lso.exe cadence-ab`: an offline diagnostic that replays already-recorded JSON reports'
  `datums` with an artificially reduced pre-groove sampling cadence and compares the resulting
  gates/trajectory/grade to the full cadence actually recorded, over a file or a directory
  searched recursively. Purely a measurement tool for the still-open adaptive-cadence question
  (B.2 of the notation/cadence work); it never changes live recording, the fork, or the input
  files.
- Source-buffered `RecoveryTelemetry` acquisition with idempotent start/read/stop lifecycle,
  exclusive sequence cursors, full-batch processing, epoch/identity validation, explicit
  retention/capacity loss and invalid-unit diagnostics, plus unary rollback through
  `--position-source unary`.
- Optional global DCS-gRPC `X-API-Key` injection from `DCS_GRPC_API_KEY` (or the variable selected by
  `--api-key-env`), with sensitive metadata marking and no token logging.
- Independent `EventCorrelator` and `ReportPipeline` components, including additive event-stream
  status/outcome-confirmation evidence in schema-v3 JSON.
- A priority `PositionCollector`, `--positions-only` baseline mode, optional suspension of background
  detector transforms, `Skip` missed-tick scheduling and per-recovery acquisition percentiles.
- Build Git commit/dirty provenance, explicit DCS-gRPC client/server API-line compatibility, sliding
  telemetry health and additive primary/secondary causes (SQLite migration 6).
- Hook gRPC status codes and recent-evidence ring retention.

- Session/generation-aware supervision, two-second RPC/watchdog deadlines, monotonic freshness and
  explicit skew/gap diagnostics with conservative short extrapolation.
- Strict AV-8B/Tarawa and hook-aircraft/arrested-carrier pairing, slot/UCID human identity and
  session-scoped AI identity.
- Bracketed gate interpolation with `Valid`, `Late`, `Missing` and `Invalid` evidence, plus ordered
  raw `Land`/`RunwayTouch`/LQM evidence and raw hook observations.
- Separate estimated/DCS wire provenance, divergence, confidence, completeness, cause and grading
  version in structured reports and additive SQLite migrations.
- Bounded telemetry/event buffers and runtime RPC, stream, queue, IO and render metrics.
- Simplified NAVAIR-style grading from 3/4, 1/2, and 1/4 nm glideslope and lineup samples, with
  `_OK_`, `OK`, `(OK)`, `--`, `C`, `B`, and `WO` labels and numeric points.
- Neutral unknown-initiator waveoff/go-around evidence, conservative touch-and-go handling, and
  explicit pass outcome storage.
- A second PNG showing the overhead carrier pattern in the BRC frame.
- Pretty-printed JSON recovery reports with gate samples, final-approach datums, and mission time.
- Persistent `<out-dir>/lso.db` storage, automatic migrations for older databases, and pilot UCID,
  aircraft, map, UTC grade time, mission time, points, and outcome fields.
- Optional HTTP greenie board (`--web-port`) and `/api/passes` JSON endpoint.
- Session greenie board printed on shutdown.
- Richer Discord embeds with map, UTC and mission time, grade/points, outcome, gate deviations, DCS
  notation translated to plain English, wind, and groove time. Both approach and pattern PNGs are
  attached; ACMI is attached unless disabled.
- `--no-acmi` to keep charts and JSON without saving Tacview recordings.
- `--no-acmi` now also skips TacView serialization and ACMI-only metadata/unit RPCs instead of only
  suppressing the final file.
- Discord "Why This Grade" field: a short, plain-language explanation of the specific rule that
  produced the pass grade (e.g. "(OK): drifted 0.6° high on glideslope — OK needs better than
  0.5°."), or the existing telemetry-unavailability message when grading itself was unavailable.
  Built by new `grade_from_gates_with_reason`/`compute_pass_grade_with_reason` (`src/grading.rs`),
  which `grade_from_gates`/`compute_pass_grade` now wrap, so the displayed reason can never diverge
  from the actual grade (same single-source-of-truth rationale as the `wire_estimated`/
  `wire_estimation` fix above). Stored on `TrackResult::grade_reason`. Replaces the separate
  "Technical status" field, which is now folded into this one.
- Discord "LSO Notes" fallback (`describe_measured_deviations`, `src/grading.rs`) for a touch-and-go,
  which DCS never sends a `LandingQualityMark` comment for (confirmed live 6 September 2026: 3 of 3
  T&G passes in one session had no `dcs_grading` at all). Explicitly labelled as measured by LSO,
  never phrased to resemble a DCS/NATOPS comment.
- Diagnostic logging for the next live investigation round: `wire_estimate_at` confidence/reason,
  3/4 NM gate eligibility, groove/touchdown timing summary, and the roll-out bank/track-angle check
  are now logged (DEBUG/TRACE); a geometric `Bolter` refused by `GRADE:WO` is now logged (INFO); a
  new `ActivePriorityPlanes::active_count` surfaces genuine concurrent-recovery overlap (INFO) when
  `--suspend-detectors-during-recovery` is set; the two false-start abandon paths in
  `record_recovery` are promoted from DEBUG to INFO with elapsed time and lowest altitude reached.

### Fixed

- `wire_estimated` could diverge from the diagnostic `wire_estimation` when the `Land` event
  correlation raced ahead of the positional wire-crossing update that produced the same value;
  `cable_estimated` is now always reconciled once, in `Track::finish()`, against the full
  wire-crossing history.
- `trajectory_deviations`' `atan2`-based GS/lineup angles could explode as the remaining distance
  to the deck approached zero: samples below `TRAJECTORY_MIN_DISTANCE_M = 3 m` are no longer pushed
  at all, and below `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75 m` the angle is computed against that
  fixed reference distance instead of the shrinking true distance — an ordinary few-decimetre
  flare/reference offset near the ramp no longer manufactures a many-degree deviation, while a real,
  larger offset still degrades the grade or trips the Cut as before.
- A go-around that overflew the deck well above deck level was classified `Bolter` instead of
  `WaveoffUnknown`; `crossed_deck_threshold` now also requires the aircraft to be near deck level
  (`DECK_CROSSING_ALT_CAP_FT = 50 ft`) at the moment of crossing.
- Confirmed live 5 September 2026 (evening, human test): the 50 ft guard above still let a
  purely geometric deck crossing with no confirmed contact and no DCS event at all (survol
  measured at 8.7 m) be classified `Bolter`. The geometry-only `Bolter` path (no event ever
  correlated) now additionally requires hook altitude at the crossing to be at or below
  `DECK_CONTACT_CONFIRMATION_ALT_M = 1.0 m` (`deck_crossing_confirmed_contact`); otherwise the
  pass is `WaveoffUnknown`. Separately, `Track::finish()` now refuses a `Bolter` outright when the
  DCS LQM itself opens with `GRADE:WO` (`dcs_grade_is_waveoff`) — not inventing a waveoff author,
  only declining a bolter DCS's own grading directly contradicts, as observed on a pass graded
  `GRADE:WO ... WO(AFU)IC` by the LSO with no `runway_touch`/`land` event.
- F-14 (all variants) hook geometry (`F14_HOOK`, `src/data.rs`) modelled the hook ~0.8-1.1 m below
  actual deck level while the aircraft was physically on deck/rolling through a wire (T-45 reads
  ~0.0 m at touchdown by comparison). Corrected with an empirical `+1.0 m` vertical offset on top
  of the ModelViewer2-extracted position, pending a fresh ModelViewer2 remeasurement and human
  confirmation on F-14A/F-14B specifically (see `tasking-roadmap.md`). This bias was feeding a
  spurious "hook below deck ⇒ contact" reading on the item above, inflating near-deck GS
  deviations, and is the likely common cause of the wire-estimate bias tracked separately.
- Hook-position calibration (`calibrated_hook_state`) could read a raw sample taken while the
  crosse was physically pressed against the deck (reads `0.0`, i.e. "up") as evidence, because the
  cut-off used to select which samples to interpret was the event-correlated `landing_time`, which
  lags true physical contact by ~0.2-1 s. On a confirmed live T-45 trap this window alone would
  have produced an invented `TouchAndGo` on a real arrest without the DCS LQM as a safety net.
  Interpretation is now frozen at the first *geometric* hook contact
  (`first_hook_ground_contact_time`, `alt <= 0.0`), whichever of that or `landing_time` fires
  first.
- `wire_estimate_at` (`Track::finish()`) could report `confidence: "high"` from tight timing
  brackets alone, even though a tight bracket only proves the crossing was measured precisely, not
  that the aircraft actually stopped there — confirmed live 5 September 2026 (evening) on
  survols/bolters with no arrest at all reading a semantically false "high" wire estimate. `"high"`
  now additionally requires a DCS-confirmed arrest (a parsed LQM `WIRE#`); without one, confidence
  is capped at `"medium"`. The other half of the same live-confirmed bias — the estimator retaining
  the highest-numbered wire crossing recorded before a late `Land`/`runway_touch` event — remains
  unresolved: correlating on `first_hook_ground_contact_time` instead was tried and reverted, since
  it discarded a still-correct wire-4 crossing on a clean straight-in trap (`wire_4_01_FA18C`) where
  the hook geometrically swept past wires 1-3 while still airborne, before the later, only-genuine
  contact. See `tasking-roadmap.md` for the decision this leaves open (likely needs a deceleration
  proxy, e.g. `touchdown_horizontal_speed_mps`, rather than a hard first-contact cutoff).
- Event-stream errors and clean closure no longer become positional `telemetry_gap`; existing gates
  remain intact while outcome availability is assessed separately.
- Plane/carrier respawns with a changed ID abort every stale same-name task within the current
  session/generation, preventing old-ID collectors from polling a current name.
- DCS/LQM wire parsing accepts only cables 1-4 and rejects zero, overflow and malformed suffixes.
- JSON, ACMI and rendered files use atomic create-if-absent publication on Windows and Unix; the JSON
  winner alone may continue to SQLite/render/Discord, and temporary files/directories are cleaned.
- Positions-only ignores missing or invalid Discord user configuration and does not start event,
  hook, ACMI, SQLite, dashboard, render, board or Discord components.
- Errors now retain useful IO paths, JSON line/column data and underlying JSON, SQLite, rendering,
  ACMI and Discord causes.
- Git dirty provenance now intentionally covers tracked files only, with tracked-path/index/HEAD
  rebuild triggers and deterministic parser tests; untracked files and `target/` are excluded.
- CI uses locked build/tests, all-target Clippy with warnings denied, rustfmt, and a pinned locked
  `cargo-audit` installation that consumes the existing `.cargo/audit.toml` ignore list unchanged.
- Detector suspension is scoped to the aircraft already being collected, so a second aircraft can
  still start a simultaneous recovery; positions-only no longer opens or migrates SQLite.
- Unconfirmed arrest no longer overwrites telemetry/gate causes; all independent unavailability
  causes are retained and SQLite completeness values now use the JSON snake-case vocabulary.
- Hook gRPC codes use documented snake-case names, baseline manifests are strict, Git dirty-state
  rebuild tracking covers every tracked file, and acquisition percentiles use bounded online
  histograms instead of unbounded vectors and end-of-pass sorting.
- Positions-only skips TacView update construction, and the direct Axum dependency is aligned with
  Tonic's 0.8 dependency line.
- Repaired the malformed Discord block left by the previous merge.
- Hook/event diagnostic truncation no longer changes positional completeness or masks
  `insufficient_gates`; hook history retains the newest 512 observations.
- Wire crossings are segmented at final entry, DCS/LQM wire evidence remains visible when the Rust
  estimate is unavailable, and each invalid gate displays its own bracket gap.
- Additional F-14 type aliases and distinct F-14A, F-14B, and F-14B(U) display names.
- Carrier-position EMA smoothing for final-approach geometry.
- Independent, timestamped hook sampling with configurable 2-4 Hz cadence, 250-300 ms timeout and
  a legacy-inline A/B switch; per-RPC and loop/tick latency percentiles; live telemetry health.
- Schema-v3 report evidence for hook freshness, component versions, grading availability and
  continuous wire-plane crossings; additive SQLite migration version 5.
- Recovery-monitor tasks for respawned units replace stale tasks instead of accumulating duplicate
  recordings after mission changes.
- Recording ends when a plane exits the 3.5 nm / 1,100 ft pattern envelope, preventing indefinite
  ACMI capture after a missed approach or mission change.
- Carrier-position smoothing reduces periodic sawtooth artifacts in final-approach charts and gate
  measurements.
- CATOBAR charts select the latest continuous inbound branch, preventing earlier overhead-pattern
  points from joining the real final as a false vertical drop.
- F-14B(U) identification and trap-sheet naming.
- Gate brackets use only their actual endpoint interval and can recover from an isolated degraded
  sample without crossing a real cut; frozen DCS timestamps now age and trip the watchdog.
- Pattern-only gaps no longer invalidate the scored groove, while gate/groove gaps remain blocking.
- Fragmented CATOBAR grooves render as separate labelled fragments instead of disappearing or being
  connected artificially. A late RunwayTouch transform can no longer manufacture a wire-4 crossing;
  an estimate now requires a continuous crossing correlated within 300 ms of the event.

### Changed

- CATOBAR groove-entry roll-out check (`is_rolled_out`, `src/track.rs`): the ground-track-angle
  proxy for "tracking down the groove axis" now fits a least-squares linear regression of position
  against time over the whole buffered `gate_samples` window (`track_velocity_regression`), instead
  of comparing only the two endpoint samples of that buffer. A single noisy or skewed telemetry
  frame sitting right at either edge of the window can no longer swing the whole estimate on its
  own, since every sample in the window now contributes to the fit. Precision improvement only:
  `GROOVE_ROLLOUT_MAX_TRACK_ANGLE_DEG`/`GROOVE_ROLLOUT_MAX_BANK_DEG` and the underlying doctrine
  (NAVAIR 00-80T-105 groove entry as a "roll wings level" event, not a fixed distance) are
  unchanged; no new `PROJECT-DERIVED` threshold introduced. An "on speed" (AoA) proxy was
  considered and explicitly rejected: this project's AoA is only a geometric approximation, never
  a value exposed by DCS-gRPC, and was judged too unreliable to gate groove-entry detection on.
- CATOBAR `_OK_`/`OK`/`(OK)` no longer unconditionally requires the 3/4 NM gate
  (`GateDeviations::all_valid`/`three_quarter_counts`, `src/track.rs`; `grade_from_gates`,
  `src/grading.rs`): when that gate was captured *before* roll-out-confirmed groove entry
  (`Track::groove_entry_time`) -- present or not, valid or not -- it counts toward neither
  completeness nor GS/lineup amplitude, and only the 1/2 NM and 1/4 NM gates (plus the continuous
  trajectory) are required/scored. On a real Case I pattern the 3/4 NM gate routinely falls in the
  base-to-final turn rather than the groove (confirmed live 5 September 2026, evening: 7 of 8 human
  passes, lineup up to -10.5° at that gate), imposing `--` regardless of the actual groove that
  followed. A 3/4 NM gate captured *after* groove entry is unaffected. V/STOL keeps the historical
  unconditional rule (it has no roll-out-confirmed groove entry to anchor this on); `cadence-ab`
  also keeps it, since it never re-derives groove entry from a replay. `PROJECT-DERIVED`; not
  revalidated on live mission data yet (no NATOPS text makes any fixed gate crossing a
  qualification requirement in the first place -- see `AGENTS.md`, "Gates, outcomes et câble").
- CATOBAR grading now takes the worst GS/lineup amplitude across the continuous trajectory as well
  as the three gates, not the three gates alone; a significant excursion strictly between two gates
  (previously invisible to grading) can now downgrade the pass, and a dip below the Cut threshold
  anywhere at or inside the quarter-NM distance is caught, not only exactly at the gate crossing.
  This can only make the reported amplitude equal or worse than before, never better.
- Pilot-facing surfaces (Discord embed, PNG chart, SQLite/greenie-board log) now always show the
  DCS/LQM wire alone when it is available, instead of ever displaying it next to a diverging Rust
  geometric estimate (`Grading::pilot_facing_outcome`); the full JSON `outcome` field still records
  both wire values side by side for diagnostics.
- Full-pattern JSON `datums` are now subsampled to one in four outside the scoring-relevant window
  (before groove entry and beyond ¾ nm / 500 ft); the scoring zone itself, gate evidence and
  grading are unaffected, only the pattern/break portion of the report shrinks.
- DCS-gRPC client stubs are aligned with the sibling `0.10.0` server checkout while its commit is
  unpublished; release packaging must replace the local path with a reviewed immutable remote pin.
- The original MOOSE-style automatic `_OK_` rule (wire-3 plus a 15-18.99 s groove-time window) was
  disabled; `_OK_` was reserved for an explicit official/manual grade for a time. A different,
  wire-independent automatic `_OK_` rule was later added — see "Added" above.
- PNG rendering and SQLite work run outside latency-sensitive sampling tasks. Atomic artifact names
  include session/generation/unit identity and database inserts are idempotent.
- The HTTP dashboard now binds to `127.0.0.1` and returns HTTP 500 on database failure.
- The detection envelope now captures the full pattern: 200 m to 3.5 nm from the carrier and at or
  below 1,100 ft MSL, without nose-pointing or rear-hemisphere checks.
- Gate sampling is restricted to inbound crossings below 500 ft above the deck, and groove entry
  also requires lineup within 10 degrees.
- The earlier DCS-gRPC migration moved client stubs to the official sevenfifty777 fork release tag `v0.9.0`,
  resolved in `Cargo.lock` to commit `5bd6d6e42491c8697a5c5a95e80a2e689923bd3b`; `tonic` was
  updated to 0.13.
- Unit discovery safely ignores DCS units whose optional type field is absent.
- T-45 AoA brackets now use values derived from the VNAO T-45 display-electronics data instead of
  the former F/A-18C copy.
- F/A-18C CQ touch-and-go recognition now requires stable, timestamped pre-touch hook evidence;
  uncalibrated modules remain unknown. Technical unavailability is separate from pilot performance.
- Discord "Gates (GS / LU)" field now shows degrees instead of feet, matching every other angle
  shown in the embed.

### Security and dependencies

- The lockfile was refreshed for DCS-gRPC 0.9.0 and its gRPC stack.
- Known vulnerable transitive versions identified during the migration were updated; the audit was
  clean at that time. Current `cargo audit` allowances are tracked directly in `.cargo/audit.toml`,
  not duplicated here since they change independently of this migration.

## 0.2.0 - 2024-11-10

- Initial tagged `0.2.0` release. It provided live DCS-gRPC monitoring, carrier-relative recovery
  charts, wire estimation, compressed ACMI recording, Discord webhook delivery, and ACMI replay.

Earlier tags: `0.1.1`, `0.1.0`.
