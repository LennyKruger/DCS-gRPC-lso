# Grading and outcome reference

LSO produces a project training score. It is never a USN/USMC certification or an official LSO
grade. Every executable rule below is labelled `OFFICIAL` or `PROJECT-DERIVED`.

## Sources and provenance

`OFFICIAL` references used for vocabulary and symbols:

- NAVAIR 00-80T-104, 1 May 2009, section 6.3.2 for qualification touch-and-go terminology,
  section 6.6.4 for foul-deck waveoff context, and section 11.4.1 for grade symbols;
- NAVAIR 00-80T-111, 15 December 2004, chapter 23 and cards A-5/A-9 for V/STOL phases and the
  human assessment of hover, cross, VL, power, attitude, spot and relative heading.

These publications do not prescribe this module's three-gate formulas, geometric thresholds or
V/STOL A/B/C/D bonus. Those rules are `PROJECT-DERIVED`.

## Structural result

The persisted result separates outcome, display grade, optional points, comment/cause, confidence,
completeness, grading version, cable estimate and DCS cable evidence.

An incomplete observation has grade `NC` and `points = null`. `WO?` means a go-around/waveoff was
observed but its initiator was not proven. The module never invents OWO, WOP or a pilot waveoff.

### What `NC` actually means

`NC` is a single project display symbol, but it is never a single internal reason. The JSON
`cause`/`causes` fields (`docs/DATA_CONTRACTS.md`) already separate "telemetry too degraded to
measure" from every other kind of unavailability, so a diagnostic consumer never has to guess which
applies:

| `cause` value | Meaning | Category |
|---|---|---|
| `telemetry_gap` | A gap in the scored segment exceeded the sample-gap/extrapolation limits | Telemetry too degraded to measure |
| `invalid_telemetry` | A scored sample failed validation (skew, non-monotonic time, etc.) | Telemetry too degraded to measure |
| `position_buffer_limit` | The position buffer overflowed or lost samples in the scored segment | Telemetry too degraded to measure |
| `insufficient_gates` | Telemetry was fine, but fewer than three valid, ordered gates were captured | Structural — nothing to grade |
| `unconfirmed_arrest` | Contact was observed but no DCS/LQM wire confirms an arrest | Proof, not measurement, is missing |
| `unknown` (grading only, `Grading::Unknown`) | The aircraft was tracked but never became a scored approach | Not a telemetry problem at all |

A pass can report more than one of these at once: `cause` is the highest-priority one (see
`Completeness::priority` in `src/track.rs`) and `causes.secondary` lists the rest. Because this
differentiation already exists in the persisted JSON, no additional internal status or field was
introduced: `NC` itself never needs to distinguish these cases, since anything that needs to is
already reading `cause`/`causes`, not the display grade.

## Gates

`PROJECT-DERIVED`, `project-derived-v4`:

| Gate | Distance | Acceptance |
|---|---:|---|
| 3/4 NM | 1,389 m | valid inbound bracket, ordered/fresh/skew-valid samples |
| 1/2 NM | 926 m | same |
| 1/4 NM | 463 m | same |

At threshold `x`:

```text
ideal_alt = base_alt + x * tan(aircraft_glide_slope)
gs_deg    = atan2(observed_alt - ideal_alt, x)
lineup    = atan2(lateral_offset, x)
```

Both bracket endpoints must be valid, in phase and lined up. A bracket gap above 300 ms, skew above
300 ms, or non-increasing DCS time invalidates the gate. Starting inside a gate records `Late` and
does not manufacture a historical observation. The three valid gate times must be strictly ordered.

## CATOBAR score

All numerical boundaries and point mappings in this table are `PROJECT-DERIVED`, retained from the
historical module/MOOSE-inspired model pending validation. `abs(GS)`/`abs(LU)` are the worst values
found across the three gates **and** the continuous trajectory (see below):

| Result | Rule | Points |
|---|---|---:|
| `_OK_` | `OK` (below), tightened to `abs(GS) <= 0.4/0.3 deg` (high/low) and `abs(LU) <= 0.5 deg` everywhere, **and** groove time `15-18 s`; never for a touch-and-go | 5.0 |
| `OK` | all three gates valid; `abs(GS) < 0.5 deg`, `abs(LU) < 1.0 deg` | 4.0 |
| `(OK)` | no significant deviation; `abs(GS) >= 0.5 deg` or `abs(LU) >= 1.0 deg` | 3.0 |
| `--` | `abs(GS) >= 1.0 deg` or `abs(LU) >= 2.0 deg` | 2.0 |
| `C` | GS strictly below `-2.5 deg` at the quarter-NM gate, or anywhere in the continuous trajectory at or inside 463 m; or a sustained (>=3 consecutive samples) sink rate `>= 8.0 m/s` or bank `>= 30 deg` inside 463 m | 0.0 |
| `B` | confirmed bolter and all three gates valid | 2.5 |
| `WO?` | neutral waveoff/go-around, initiator unknown | none |
| `NC` | insufficient/invalid telemetry or unconfirmed trap | none |

`OFFICIAL`: `_OK_` is a documented grade symbol in NAVAIR 00-80T-104 §11.4.1 ("Perfect pass"), and
the "15 - 18 second groove" duration is documented text in NAVAIR 00-80T-105 §6.2.4.3. Everything
else about how `_OK_` is calculated — the amplitude band, the exact groove-time window used, and
the decision to gate on groove time at all — is `PROJECT-DERIVED`; see "Automatic `_OK_`" below for
the full rule and its provenance.

### Continuous trajectory (amplitude only)

`PROJECT-DERIVED`. Historically only the three point-in-time gates fed the grade, so a deviation
spike strictly between two gates (e.g. between 1/2 NM and 1/4 NM) could go completely unscored even
though the full trajectory was already being recorded. `trajectory_deviations` (additive JSON field)
now samples the same GS/lineup geometry as a gate crossing, but continuously, at the aircraft's own
distance, from groove entry to touchdown. Its worst GS-high, GS-low and lineup values are combined
with the three gates' (the maximum of both), and any of its samples at or inside the quarter-NM
distance is checked against the Cut threshold exactly like the quarter-NM gate itself. This can only
ever make the amplitude reading equal or worse than the three-gate-only computation, never better,
and availability is still governed exclusively by `gates.all_valid()` — an incomplete pass is never
made gradable by trajectory data alone.

AoA is chart information only and no AoA table changes the grade. Power, wind, weight and LSO calls
are still not scored because no validated per-aircraft rule has been adopted. Sink rate and bank are
likewise not scored on amplitude/trend the way GS/lineup are — only their own dedicated danger cut
applies (see "Sink rate and bank" below).

#### Persistence guard against a single aberrant frame

`PROJECT-DERIVED`. The continuous trajectory samples telemetry every frame in the groove, which
makes it more exposed than a gate crossing (already bracket/skew-validated) to a single noisy
frame. A GS or lineup value from the continuous series only counts toward `worst_gs_high`/
`worst_gs_low`/`worst_lu` once it crosses its own `*_SLIGHT` threshold on
`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES` (2) consecutive samples in the same direction; a lone spike
that never repeats on an adjacent sample is dropped. This guard applies only to the general
amplitude tiers above — never to the Cut safety check (any single sample at or inside 463 m below
`GS_CUT_LOW_DEG` still triggers `C` immediately) or to the late-approach weighting below, both of
which stay maximally sensitive to a single dangerous sample by design.

#### Near-touchdown geometry

`PROJECT-DERIVED`. `gs_deg`/`lineup` are `atan2(offset_m, x)`: as `x` (distance-to-ship) shrinks
toward touchdown, the same offset in metres is divided by a smaller and smaller number, so the
angle grows even when the underlying offset is not getting any worse. Two distinct guards address
two distinct severities of this effect, both in `src/track.rs`:

- **`TRAJECTORY_MIN_DISTANCE_M` (3 m)**: below this floor, no sample is pushed to
  `trajectory_deviations` at all. Confirmed live on 5 September 2026: an ordinary few-decimetre
  flare produced 70.3 deg at `x = 0.30 m` and 32.5 deg at `x = 1.47 m` — angles with no geometric
  meaning, an outright numerical blow-up as `x -> 0`.
- **`NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` (75 m)**: a much more moderate, still-misleading version of
  the same effect persists well above the 3 m floor. Confirmed live the same day (see
  `tasking-roadmap.md`, 5 September 2026 evening session): on two otherwise-clean passes (one
  independently graded `_OK_` by DCS itself), an essentially **constant** few-decimetre-to-one-metre
  offset — present from at least 50 m out, both vertically and laterally, not a growing deviation —
  still ballooned to a double-digit-degree reading in the final few metres purely from the shrinking
  denominator, pulling an otherwise `Ok`/`(Ok)`-grade pass down to `NoGrade`. Conversely, on a
  genuinely poor pass, a real, roughly stable ~1.4-1.7 m low bias present from ~110 m inbound only
  crossed the `-2.5 deg` Cut threshold once `x` shrank below ~31 m — the Cut fired from geometry
  catching up with an old, non-worsening deviation, not from anything happening near the ramp.

  Fixed by substituting a fixed reference distance for `x` in the `atan2` call once the real `x`
  drops below it (`x.max(NEAR_TOUCHDOWN_ANGLE_REFERENCE_M)`, `trajectory_deviation_angles_deg` in
  `src/track.rs`, shared by `Track::next` and `replay_gate_and_trajectory`): every sample farther
  out than the reference distance is untouched (`max` is a no-op once `x` already exceeds it), and
  every closer sample now reports an angle proportional to its real offset in metres rather than to
  that offset divided by an almost-zero remaining distance. A genuinely large offset is still just
  as able to cross a tier or Cut threshold — `atan2(-3.5 m, 75 m) ~= -2.67 deg`, still a Cut — it
  just now takes a real number of metres of deviation to do so, not merely a few decimetres
  combined with being close to the ramp. This changes only *how the angle for a near-touchdown
  sample is computed*; the "the continuous trajectory can only ever worsen the amplitude reading,
  never improve it" rule above, and the Cut/late-window/persistence sensitivities, are unchanged.

  `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75.0` is `PROJECT-DERIVED`, chosen as roughly one second of
  flight at a typical CATOBAR approach speed (~75 m/s) — comfortably larger than the confirmed-live
  flare/reference offsets divided by the `Ok`/Cut angular thresholds, while short enough that a
  deviation only developing in the true final seconds is still evaluated with its own reasonably
  real geometry rather than one held all the way back to a gate distance. Not yet validated against
  a wider live corpus (only the 5 September 2026 evening session informed this constant) — see
  `tasking-roadmap.md`.

  **A related, separate observation, not addressed by this fix**: the near-constant lateral offset
  itself (~0.75-0.85 m, present at every distance from at least 50 m out to touchdown, on every one
  of the five recoveries observed that evening — both clean and Cut passes) is odd enough to be
  worth investigating on its own. It does not scale with `x` the way a real angular lineup error or
  an axis-alignment error would (both would produce an offset roughly *proportional* to `x`, not a
  constant one), so it looks more like a fixed reference-point discrepancy (e.g. hook-versus-CG
  projection, or the assumed touchdown aim point) than a real, growing lineup deviation. Flagged in
  `tasking-roadmap.md` for a future session; this fix's reference-distance substitution reduces its
  effect on the grade regardless of its ultimate cause, but does not explain or correct it.

### Automatic `_OK_` ("Perfect pass")

Implemented 5 September 2026, in `is_amplitude_perfect`/`grade_from_gates` (`src/grading.rs`).
`_OK_` is reachable **only** from a pass the rules above have already independently graded `Ok`
(every tier, trend, oscillation and late-window check already clear) — it is a strict tightening of
`Ok`, never an alternate path, and it can only ever raise `Ok` to `_OK_`, never anything else to
`_OK_` directly.

Two conditions, both required:

1. **Amplitude**: every gate, and every continuous-trajectory sample, stays within
   `OK_PERFECT_GS_HIGH_DEG`/`OK_PERFECT_GS_LOW_DEG` (`+0.4`/`-0.3 deg`, asymmetric) and
   `OK_PERFECT_LU_ABS_DEG` (`0.5 deg`). Gates are trusted, bracket/skew-validated single points —
   a single gate outside the band denies `_OK_` outright, no pardon. The continuous trajectory
   gets the same noise pardon as everywhere else (`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`, 2): one
   isolated, non-repeating frame outside the band does not by itself deny `_OK_`, but two
   consecutive samples do.
2. **Groove time**: `groove_time_secs` must be `Some` and fall in
   `OK_PERFECT_GROOVE_TIME_MIN_S..=OK_PERFECT_GROOVE_TIME_MAX_S` (`15.0..=18.0`, inclusive both
   ends). `None` (not recorded) denies `_OK_` — evidence is never assumed absent. Since
   5 September 2026 this value is also serialised into the JSON report (`groove_time_secs`, see
   `docs/DATA_CONTRACTS.md`) — before that it was computed but only ever surfaced in the optional
   Discord embed, so a live report with Discord disabled had no way to audit `_OK_` eligibility
   after the fact.

**Provenance, precisely**: the `_OK_` *symbol* and its meaning ("Perfect pass") are `OFFICIAL`
(NAVAIR 00-80T-104 §11.4.1). The groove-time window is also `OFFICIAL` text (NAVAIR 00-80T-105
§6.2.4.3: "a 15 - 18 second groove before aircraft touchdown"), but NATOPS never ties that duration
to an automatic grading decision — using it to gate `_OK_` is this module's own choice. The
amplitude band is fully `PROJECT-DERIVED`: NATOPS gives no numerical criterion for "perfect" at
all. It is borrowed from a real, independently-maintained open-source LSO grading implementation —
MOOSE `Ops.Airboss` (`Airboss.lua`, `AIRBOSS.GLE`/`AIRBOSS.LUE` `_max`/`_min` fields) — the same
historical/MOOSE lineage the module's other `GS_SLIGHT_*`/`LU_SLIGHT` thresholds already descend
from, tightened to the band Airboss itself reserves for a zero-deviation "Unicorn" pass.

**What was deliberately not revived**: the historical MOOSE "Unicorn" rule additionally required
wire 3 specifically, and a narrower ad-hoc time sub-window layered on top of the NATOPS figure.
Neither NATOPS document ties any wire number to a grade, so this module's rule depends on groove
time alone — wire 3, wire 1, or wire 4 all reach `_OK_` identically given the same amplitude and
groove time. A touch-and-go is capped one tier below whatever `grade_from_gates` computes (so a
textbook-perfect touch-and-go still stops at `Ok`, never `_OK_`): a touch-and-go is a deliberate
hook-up qualification pass, never a full stop, and "Perfect pass" is reserved for a real trap.

**A known, accepted limitation**: the 15-18 s window describes a standard CATOBAR jet groove at
its normal approach speed. The VNAO T-45 flies a different glide slope (3.0° vs 3.5°) and likely a
different real approach speed that this module has no per-type reference for, so the same raw
window is applied to it unadjusted. Not yet validated on any live recording for any type — see
`tasking-roadmap.md`.

**Diagnostic-only gap**: `lso.exe cadence-ab`'s replay path (`src/commands/cadence_ab.rs`) never
computes a groove-time span (it does not replay touchdown detection at all, by design — see its
own module documentation), so it always passes `None` for groove time and can therefore never
produce `_OK_` in a replayed grade, only up to `Ok`. This affects only the offline diagnostic, not
live grading.

### Correction trend (Ok vs (OK) only)

`PROJECT-DERIVED`. NATOPS 00-80T-104 section 11.4.1 distinguishes `OK` ("reasonable deviations
**with good corrections**") from `(OK)` ("fair — reasonable deviations") on whether corrections
were good, not on amplitude alone — a distinction the amplitude-only rules above cannot express:
two passes with identical worst deviations, one that arrived high and corrected to clean, one that
arrived clean and drifted to the same worst value, would otherwise be graded identically.

Once amplitude alone would already grade a pass `Ok` (every trajectory sample and gate under the
`GS_SLIGHT_HIGH`/`GS_SLIGHT_LOW`/`LU_SLIGHT` thresholds), a simple two-point slope of `abs(GS)` and
`abs(lineup)` is computed over the final 4 seconds of the recorded trajectory (`TREND_WINDOW_S`,
chosen because a correction takes roughly 1-2 s to fly, so 4 s gives room to see a real trend
without reaching into an unrelated earlier part of the approach — deliberately no more
sophisticated filtering than that, per the design brief for this item). If either slope reaches
`TREND_WORSENING_DEG_PER_S` (0.075 deg/s — chosen so the check is reachable at all within the Ok
amplitude ceiling over that window, while staying clearly above ordinary aim-point noise; not a
NAVAIR value), the pass is capped at `(OK)` instead of `Ok`.

Trend is asymmetric by design: it can only ever **hold back** an `Ok` that amplitude alone would
have granted, never **raise** a grade amplitude alone placed below `Ok`, and it is never checked at
all once a pass is already below `Ok` — matching the same "never invent leniency" rule already
applied to the continuous trajectory itself. Whether a deviation was corrected within the window,
rather than only whether it grew, is not evaluated; nor is duration of a given deviation. Both would
require more than the two-point derivative this item's design brief called for.

### Overcontrol / oscillation (Ok vs (OK) only)

`PROJECT-DERIVED`. The correction-trend check above only ever looks at the *net* slope between the
start and end of its window, so a pilot correcting back and forth around the aim point
(`+0.4°/-0.4°/+0.4°/-0.4°`, for example) can show a near-zero net slope — and therefore pass the
trend check — while still exhibiting exactly the alternating, over-controlled piloting NATOPS
00-80T-104 penalizes under its `OC` (overcontrolled) call.

Once amplitude and trend both would already grade a pass `Ok`, the same final `OSCILLATION_WINDOW_S`
seconds of the trajectory (reusing `TREND_WINDOW_S`'s own rationale) are scanned for direction
reversals in the signed GS and lineup series: a step smaller than `OSCILLATION_MIN_SWING_DEG`
(0.3 deg) is treated as noise and never counted as a leg, and the reference point only advances on a
significant step, so a run of sub-threshold jitter cannot mask a real swing. If either series shows
at least `OSCILLATION_MIN_REVERSALS` (2) reversals — the minimum shape (out, back, out again) that
distinguishes a genuine oscillation from a single correction overshoot — the pass is capped at
`(OK)` instead of `Ok`. Same asymmetric contract as the trend check: only ever holds `Ok` back,
never raises a grade, and is never checked once a pass is already below `Ok`.

### Late-approach weighting (Ok/(OK) vs NoGrade only)

`PROJECT-DERIVED`. The same magnitude of GS/lineup error is more dangerous the closer it happens to
the ramp, because there is less distance left to correct it before touchdown — the rationale the
project's own design discussion gives for weighting the final moments of the approach more heavily.

Rather than reweighting `worst_gs_high`/`worst_gs_low`/`worst_lu` above (which would also rescale
the three named gates, whose `distance_m` values are exact, and complicate every existing
threshold), this is implemented as a separate, narrower, downgrade-only check: if any continuous
trajectory sample at or inside `LATE_WINDOW_DISTANCE_M` (150 m — deliberately narrower than the
463 m quarter-NM Cut gate, so this only ever tightens the very end of the approach the Cut rule
already treats specially) crosses `LATE_WINDOW_GS_DEG` (0.8 deg) or `LATE_WINDOW_LU_DEG` (1.5 deg) —
values deliberately set between the general `*_SLIGHT` and `*_SIGNIFICANT` thresholds — a pass that
amplitude and trend would otherwise grade `Ok` or `(OK)` is capped at `--` instead. It never raises a
grade, never affects an already-`NoGrade`-or-`Cut` result, and never touches Cut itself, which
already returns before this check runs.

Two passes with the *identical* deviation amplitude are therefore not necessarily graded the same:
one earns `(OK)` if it happened at 700 m with room left to correct, the other is capped at `--` if
the same reading happened inside the last 150 m before the ramp.

### Wind

`PROJECT-DERIVED`, contextual only. `wind_heading_deg`/`wind_speed_mps` (additive JSON fields) are
queried once per recovery from `AtmosphereService.GetWind` at the carrier's last known position, so
a report can show that a deviation happened in a stiff crosswind rather than calm air. Both fields
are absent when the query fails or in `--positions-only` (which never queries output-only DCS
metadata). **Wind never changes `pass_grade` or `grade_points`**: the project has no validated
doctrine for how much correction credit a given wind condition should earn, so inventing one here
would be exactly the kind of unverified rule this module otherwise avoids.

### Sink rate and bank

`PROJECT-DERIVED`. `TrajectoryDeviation` (the continuous groove-to-touchdown series) carries
`alt_m` (deck-relative altitude at the sample), `sink_rate_mps` (rate of altitude loss since the
previous continuous-trajectory sample, positive = descending, `0.0` for the first sample of a run)
and `bank_deg` (raw telemetry roll). `Datum` (the full recorded trajectory) carries the same raw
roll as `roll_deg`, so the `cadence-ab` replay path (see `AGENTS.md`) can reconstruct `bank_deg`
identically from a persisted report.

Both now also feed a **danger cut**, `dangerous_sink_rate_or_bank` (`src/grading.rs`): a sustained
excessive sink rate (`SINK_RATE_CUT_MPS = 8.0 m/s`) or bank angle (`BANK_ANGLE_CUT_DEG = 30 deg`)
at or inside the quarter-NM gate — the same "no distance left to correct it" zone as
`GS_CUT_LOW_DEG` — grades the pass `C`, exactly like a dangerously low glideslope. The run must be
at least `DANGER_CUT_MIN_CONSECUTIVE_SAMPLES` (3) consecutive samples, a stricter guard than the
ordinary amplitude persistence check (`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`, 2), since Cut is the
harshest verdict available (0 points) and must never trigger on a single noisy telemetry frame.

**Neither threshold is NATOPS-numeric**, unlike `GS_CUT_LOW_DEG`: NAVAIR 00-80T-104 §6.6.4 and the
`TMRD`/`W`/`TMA`/`DLW`/`DRW` comment codes (NAVAIR 00-80T-105) leave excessive sink rate and bank
entirely to the controlling LSO's judgment, never a chargeable number. `SINK_RATE_CUT_MPS` is set
at roughly double the publicly documented nominal CATOBAR no-flare approach/touchdown sink rate
(~600-800 ft/min, ~3.0-4.1 m/s), so a normal, intentional touchdown never trips it while a real
dive or late correction well outside that corridor does; `BANK_ANGLE_CUT_DEG` is set at double
`GROOVE_ROLLOUT_MAX_BANK_DEG` (`src/track.rs`, 15 deg, the CATOBAR groove roll-out "wings level"
threshold). Neither number has been validated against a live recording — see
`tasking-roadmap.md`, "Décisions encore ouvertes".

### AoA

`PROJECT-DERIVED`, chart/report context only — **never scored**, and this section does not change
that. Real LSO doctrine does weigh AoA/airspeed as one of several judgment factors (NATOPS lists it
among the glideslope techniques and the trend-form categories), so the module aims for a value that
is at least directionally honest, even though it does not attempt to reproduce a cockpit gauge's
exact calibration.

The `aoa` value in `datums`/`pattern_datums` was originally the angle between the aircraft's nose
and its raw **ground-relative** velocity vector — a proxy that is systematically wrong whenever
there is wind (i.e. on every carrier recovery, since wind-over-deck is standard procedure: 20-25 kts
against a ~130-140 kt approach speed is not a rounding error), and that mixes true vertical AoA with
lateral sideslip/crab into one combined, always-positive angle.

Once a wind reference is available for the recovery (see below), `aoa` instead:

1. subtracts the interpolated wind vector from ground velocity to approximate true airspeed;
2. transforms that vector into the aircraft's own body frame;
3. keeps only the vertical (pitch-plane) component, discarding the lateral one — the same
   decomposition a real vane-type indicator performs by construction, and signed (unlike the old
   value) so a nose-below-flight-path moment reads as a small/negative angle instead of an
   unsigned magnitude.

**Wind reference.** DCS wind is deterministic and does not vary over time once a mission is
running — only with altitude, per the two-layer profile (ground level / 2000 m) exposed in the
mission editor. So instead of polling continuously, two `AtmosphereService.GetWind` calls are made
once, at groove entry: one at the aircraft's current altitude, one near the deck. Every subsequent
sample's wind is linearly interpolated between these two by its own altitude
(`WindReference::at_altitude`), assuming DCS itself interpolates linearly between its two
configured layers — a reasonable but unverified assumption, since the engine's own interpolation
curve was not inspected. `wind_reference_established` (additive JSON field) records whether this
succeeded; when it is `false` (query failure, or the aircraft never entered the groove), every
`aoa` value for that recovery is still the raw, uncorrected geometric approximation — a value is
never fabricated in its place.

**What this still is not**: a reconstruction of any aircraft's actual calibrated AoA curve (which
depends on real aerodynamic data — weight, flap/slat configuration — that DCS does not expose via
gRPC). The existing `aoa_rating` threshold tables per aircraft (`src/data.rs`, sourced from public
documentation — VRS's Hornet indexer bracket, Heatblur's F-14 manual, a decompiled VNAO T-45 DEU
reference) were already being applied to the old, wind-biased value; correcting the wind bias makes
the number those tables classify more honest, but does not itself validate the tables. A real
cockpit-read AoA (via a DCS draw argument) was investigated and shelved: the argument does not
exist in the 3D model for at least the T-45C (confirmed via ModelViewer 2.0 against the client
installation), and no verifiable, primary-source draw-argument number could be found for the other
three aircraft either.

## Wire evidence

`PROJECT-DERIVED` geometry estimates the closest cable midpoint to the transformed hook/contact
position, including the historical 3 m event-latency compensation. Angles are converted from degrees
to radians before rotor construction. The estimate is the primary display wire under decision J,
but is always labelled estimated.

`wire_dcs` is parsed independently from LQM text. `wire_divergent` is true when both sources exist and
differ. Only DCS wire evidence currently confirms an arrested trap for scoring; a minimum geometric
distance or estimated cable alone does not.

## Touch-and-go vs Bolter

Both outcomes share the same trigger: distance to the touchdown point reaches a minimum, then grows
past 150 m again (`src/track.rs`, `record_correlated_touchdown`/deck-crossing paths) — the aircraft
touched, then departed without stopping. What separates them is the arresting-hook position at that
moment, read directly from DCS (`UnitService.GetDrawArgumentValue`), never inferred from behaviour
alone: hook up (raised on purpose, a deliberate CQ practice pass) is `TouchAndGo`; anything else is
`Bolter`.

`AirplaneInfo::hook_draw_argument` (`src/data.rs`) gives the draw-argument index per type:

| Type | Draw argument index | Polarity confirmed? |
|---|---:|---|
| F/A-18C Hornet | 25 | Yes — empirically confirmed (`"fa18c_zero_up_one_down_test_corpus"`) |
| VNAO T-45 Goshawk | 25 | No — user-supplied index, same `<=0.2`/`>=0.8` convention assumed |
| F-14A / F-14B / F-14B(U) | 1305 | No — user-supplied index, same convention assumed |
| AV-8B (V/STOL) | none | N/A — no arresting hook for this workflow |

`Track::calibrated_hook_state` (`src/track.rs`) interprets a raw value only for a type with a known
index: `<= 0.2` is up, `>= 0.8` is down, anything else (including no known index at all) is
`Unknown`. A stability requirement guards against a mid-animation reading: at least 3 consecutive
successful samples spanning >= 0.4 s to confirm "up", but only 2 spanning >= 0.2 s to confirm
"down" — deliberately asymmetric, since concluding "up" is what turns the conservative default
(`Bolter`) into the more favourable `TouchAndGo`, so it is held to the higher bar. Only samples taken
inside the last quarter-NM and before touchdown is recorded ever count; a hook movement after that
point can never rewrite an already-decided verdict.

**Reliability, precisely**: the underlying signal (a real hook-animation value read from the sim,
not inferred from behaviour) is sound wherever it is calibrated, and the conservative default
(`Unknown`/ambiguous/stale evidence -> `Bolter`, never a fabricated `TouchAndGo`) matches this
module's "never invent a favourable outcome" rule elsewhere. But the F/A-18C is the only type whose
`<= 0.2`/`>= 0.8` polarity has actually been confirmed against real data; the T-45 and F-14 indices
were supplied without an independent polarity confirmation, so a genuine touch-and-go on either type
could still be misclassified as a bolter (or vice versa) if the assumed convention turns out to be
wrong for that model. Not yet validated on any live recording for either type.

## V/STOL experimental score

Everything in this section except the cited doctrinal vocabulary is `PROJECT-DERIVED` and
experimental. It must not be represented as NAVAIR scoring.

Phase 1 activates only AV-8B NA on Tarawa, with `intended_spot = 7.5`. The active geometric catalog
contains only calibrated spot 7.5, so `actual_nearest_spot` is independently selected as 7.5 when a
touchdown position exists. Spots 7 and 8 are explicit future catalog candidates; they are neither
active nor scored until live calibration is available.

The approach score is the arithmetic mean of the three CATOBAR-style gate point values. Three valid
gates are mandatory. The touchdown distance on the deck plane maps to:

| Distance from 7.5 | Spot grade | Bonus |
|---:|---|---:|
| `< 1 m` | A | 1.00 |
| `>= 1 m` and `< 3 m` | B | 0.75 |
| `>= 3 m` and `< 5 m` | C | 0.50 |
| `>= 5 m` | D | 0.00 |

The bonus is capped at 5.0, then mapped to `OK >= 4`, `(OK) >= 3`, `-- >= 2`, otherwise `C`.
The 15 m zone around 7.5 records entry/presence/exit for information only. It never creates a
penalty or `foul deck` result.

The exact Tarawa event behaviour, spot geometry, wire accuracy, hook polarity and VL/RVL boundaries
remain live-validation items. Raw touchdown horizontal speed and raw hook values are retained so
those decisions can later be made without rewriting history.
