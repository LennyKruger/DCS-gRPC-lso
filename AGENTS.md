# AGENTS.md — Contexte machine-first pour DCS-gRPC-lso

> Document de continuité, à tenir à jour à chaque changement significatif de code ou de contrat.
> Dépôt `E:\DCS stuffs\Initiative ESG\DCS-gRPC-lso`, branche `feature/refonte-v3-lua-buffer`.
> Dernier état vérifié : HEAD `96fd52e` ("Corrections/Améliorations/Optimisations suite tests
> 04/09/2026") plus des modifications non committées sur `src/track.rs`, `src/grading.rs`,
> `src/data.rs` et `src/tasks/record_recovery.rs` (raffinement CATOBAR de l'entrée en groove par
> confirmation de roulis/route, correctif de la désynchronisation `wire_estimated`/
> `wire_estimation`, plancher de distance sous lequel `trajectory_deviations` n'est plus poussé pour
> éviter l'explosion `atan2` près du toucher, nouveau Cut sink-rate/bank-angle soutenu près du pont,
> `_OK_` automatique (amplitude MOOSE-inspirée + temps de groove NATOPS 15-18 s), calibration de
> la position de crosse (T&G vs Bolter) étendue au T-45 et aux F-14 via leurs propres index de draw
> argument, `groove_time_secs` désormais exposé dans le JSON (auparavant calculé mais uniquement
> visible dans l'embed Discord, donc invisible sans Discord configuré), et un second plancher
> `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` (75 m) empêchant un flare/écart quasi constant dans les
> derniers mètres d'être amplifié en un angle de plusieurs dizaines de degrés par la seule
> réduction de la distance restante — voir [tasking-roadmap.md](tasking-roadmap.md)). Crate `lso`
> 0.2.0, Rust 2021 ; les changements
> postérieurs au tag `0.2.0` sont sous `Unreleased` dans [CHANGES.md](CHANGES.md).

Pour un résumé humain, vulgarisé, du fonctionnement du module : voir [primer.md](primer.md).
Pour la roadmap, les décisions ouvertes et l'historique synthétique des sessions de travail : voir
[tasking-roadmap.md](tasking-roadmap.md).

## Règles de vérité

Ordre des sources : résultat fraîchement exécuté sur le worktree > code courant > artefact live
identifié > décision/contrat > cible de refonte > documentation historique/hypothèse. Ne jamais
conclure depuis une checklist ou un ancien résultat.

Préserver les modifications utilisateur. Ne jamais annoncer de validation DCS live, de
compatibilité fonctionnelle DCS-gRPC hors de la ligne mineure déjà validée à l'exécution, ou d'une
amélioration du p99 sans preuve nouvelle. Ne pas modifier le fork DCS-gRPC/Lua sans demande
explicite. Ne pas relever le seuil de 300 ms, interpoler une coupure proche de 900 ms, fabriquer
une trajectoire, traiter l'ACMI LSO comme source indépendante, exposer un UCID hors SQLite/API
privée, ou déplacer silencieusement le métier dans Lua.

## État exécutable vérifié

Dernière validation locale complète (5 septembre 2026, après le raffinement CATOBAR de l'entrée en
groove par roulis/route, les deux correctifs P0 confirmés en live le même jour — désynchronisation
`wire_estimated`/`wire_estimation` et explosion numérique `atan2` de `trajectory_deviations` près du
toucher —, le Cut sink-rate/bank-angle, l'automatisation de `_OK_`, l'extension de la calibration
de crosse au T-45/F-14, l'exposition de `groove_time_secs` dans le JSON et le second plancher
`NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` suite à un nouveau test live CVN-72 le même jour, voir
[tasking-roadmap.md](tasking-roadmap.md)) :

- `cargo test --locked --no-fail-fast` : **196 réussis, 0 échec** (194 tests du binaire + 2 tests
  de provenance de build) ;
- `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` propres ;
- Working tree non propre : `src/track.rs`, `src/grading.rs`, `src/data.rs` et
  `src/tasks/record_recovery.rs` modifiés et non committés (voir ci-dessus).

Aucune preuve DCS live n'est revendiquée pour l'état courant : seuls les tests automatisés et un
examen manuel du code valident les derniers changements. Voir
[tasking-roadmap.md](tasking-roadmap.md) pour le détail des runs live passés (dates, résultats,
bugs trouvés) et ce qui reste à valider en mission.

## Produit et périmètre métier

DCS-gRPC-lso est un client Rust/Tokio externe à DCS World (`lso.exe`). Il détecte les recoveries,
collecte les transforms avion/navire, construit une trajectoire relative, capture trois gates,
corrèle les événements, estime le câble, calcule un score de projet et produit les artefacts de
débrief (JSON, PNG, ACMI, SQLite, Discord, board HTTP).

- CATOBAR : F/A-18C, F-14A, F-14B, F-14B(U), VNAO T-45 sur Nimitz/Forrestal.
- V/STOL expérimental : AV-8B NA sur LHA Tarawa uniquement (voir [VSTOL.md](VSTOL.md)).
- Humains ; IA seulement avec `--ki`.
- Multi-avions/navires/recoveries isolé par session et génération.
- `lso run` live ; `lso file` rejoue seulement un ACMI créé par LSO ; `lso cadence-ab` est un
  diagnostic hors-ligne, ne rejoue rien en live.

Le grade est un score **PROJECT-DERIVED** `project-derived-v4`, jamais une certification
USN/USMC. Puissance moteur, mouvement du pont, et auteur réel du waveoff ne sont pas notés. AoA et
vent sont persistés dans le rapport (contexte uniquement, jamais notés). Sink rate (`sink_rate_mps`)
et gîte (`bank_deg`), calculés depuis la télémétrie continue, restent contexte uniquement sur leur
amplitude/tendance générale, mais alimentent chacun un Cut dédié en cas d'excès soutenu près du pont
(voir "Gates, outcomes et câble" plus bas). Ne pas modifier les règles CATOBAR/V/STOL sans demande
dédiée — voir [docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md) pour la spécification exacte à
jour.

## DCS-gRPC et dépendances

- Stubs : **dépendance de chemin local** vers `../DCS-gRPC/stubs` (`Cargo.toml`
  `[dependencies.stubs] path = "../DCS-gRPC/stubs"`), alignée sur le checkout frère
  `E:\DCS stuffs\Initiative ESG\DCS-gRPC`, workspace `dcs-grpc v0.10.0`. Ce n'est **pas** un
  tag/rev Git figé : le fork contient `RecoveryService` (start/read/stop telemetry), et LSO en
  dépend directement pour la source bufferisée par défaut. Avant tout packaging release, remplacer
  ce chemin local par un pin Git immuable et revu.
- `tonic = 0.13`, Axum direct 0.8 (aligné sur la ligne de Tonic).
- URI par défaut `http://127.0.0.1:50051`, deadline/connect timeout 2 s, retry exponentiel sans
  limite totale, intervalle max 30 s.
- Compatibilité serveur : `dcs_grpc_compatibility` est calculé à l'exécution contre le serveur
  réellement connecté ; classification `compatible_same_api_line` avec avertissement pour une
  autre version mineure de la même ligne, `incompatible` pour une autre ligne. Aucune compatibilité
  fonctionnelle au-delà de cette classification n'a été validée live avec la version actuelle des
  stubs.
- `docs/DCS-gRPC-0.9.0/` est un snapshot fournisseur historique (pré-`RecoveryService`), pas une
  preuve du déploiement actuel ; il n'est pas maintenu comme documentation LSO (voir README).

## Architecture courante

```text
DCS / Mission Scripting Environment
  -> DCS-gRPC Lua + DLL (buffer circulaire RecoveryTelemetry)
  -> superviseur session/génération + inventaire initial/Birth
  -> registre de tâches par noms, IDs, session et génération
  -> détecteur par paire compatible
  -> record_recovery
       -> PositionCollector prioritaire
       -> EventCorrelator indépendant
       -> hook indépendant ou désactivé
       -> Track / gates / trajectoire continue / santé / grading
  -> ReportPipeline
       -> JSON create-if-absent
       -> ACMI / SQLite / PNG / Discord du producteur gagnant
```

Frontières implémentées (fichiers vérifiés présents) :

- [src/tasks/position_collector.rs](src/tasks/position_collector.rs) : deux transforms
  prioritaires, alignement et métriques ; aucune dépendance événements/hook/sorties.
- [src/tasks/event_correlator.rs](src/tasks/event_correlator.rs) : identité plane/carrier, LQM,
  touchdown, disparition et état du stream ; aucune modification de la complétude positionnelle.
- [src/tasks/report_pipeline.rs](src/tasks/report_pipeline.rs) : claim par `recovery_id`,
  publication atomique JSON/ACMI/PNG, rendu temporaire nettoyé et refus de remplacement.
- [src/tasks/record_recovery.rs](src/tasks/record_recovery.rs) : orchestration restante ;
  volumineux (>1700 lignes), mais les responsabilités critiques précédentes sont testables
  séparément. Le découplage complet vers `EventCorrelator`/`ReportPipeline` prévu par la refonte
  v3 n'est pas terminé — voir [tasking-roadmap.md](tasking-roadmap.md).
- [src/track.rs](src/track.rs) : géométrie, gates, crossings, hook evidence, complétude, santé,
  outcomes, trajectoire continue (`trajectory_deviations`) et grading.
- [src/commands/run.rs](src/commands/run.rs) : connexion, inventaire, respawns, registre de
  tâches, positions-only et supervision.
- [src/commands/cadence_ab.rs](src/commands/cadence_ab.rs) : commande `lso.exe cadence-ab`,
  diagnostic hors-ligne rejouant des `datums` déjà enregistrés avec un sous-échantillonnage
  artificiel, sans jamais modifier la capture live ni les fichiers d'entrée.

Le collecteur source bufferisé Lua/DCS-gRPC **est implémenté et actif par défaut**
(`--position-source buffered`) : `PositionCollector` consomme
`RecoveryService.ReadRecoveryTelemetry` par lots incrémentaux (`after_sequence`), plutôt que deux
`GetTransform` concurrents. Le polling unary (`--position-source unary`) reste disponible comme
rollback explicite. Côté fork, `Read` purge du ring les séquences acquittées par le
`after_sequence` de la lecture suivante (jamais le lot en cours) ; le bloc `diagnostics` est
renvoyé au maximum 1×/s (`recoveryTelemetry.diagnosticsIntervalSeconds`, défaut 1.0) ; la table
`telemetryObservationErrors` est bornée à 128 entrées (FIFO) — voir `docs/recovery_telemetry.md`
du fork pour le détail à jour.

## Contrat de télémétrie

Contrat `telemetry-contract-v1`, PROJECT-DERIVED :

- cadence cible 10-20 Hz selon la source, `MissedTickBehavior::Skip` ;
- skew <=100 ms : direct ;
- 100<skew<=300 ms : extrapolation de position seulement avec historique valide/frais ;
- skew >300 ms : invalide ;
- gap/source age >300 ms : warning et bracket gate invalide ;
- gap/source age >1 000 ms : `TelemetryGap` ;
- watchdog sans progression source : 2 s ;
- reset de l'aligneur après erreur ;
- timestamps DCS, réception Unix et horloge monotone distincts.

Aucune baseline live récente n'est revalidée dans ce document — voir
[tasking-roadmap.md](tasking-roadmap.md) pour les dernières mesures live datées et leurs limites.

## Gates, outcomes et câble

- Gates : 3/4 NM 1 389 m, 1/2 926 m, 1/4 463 m.
- États : `Missing`, `Late`, `Invalid`, `Valid`.
- Validité : deux samples inbound encadrants, temps croissant, bracket <=300 ms, skew <=300 ms,
  phase/altitude admissibles.
- Trois gates valides et ordonnées sont obligatoires pour une note favorable.
- Démarrage à l'intérieur : `Late`, jamais de donnée inventée.
- Entrée en groove CATOBAR (`entered_groove`) : la boîte géométrique historique (`x <= 3/4 NM`,
  `alt <= 300 ft`, lineup `<= ±10°`) est nécessaire mais plus suffisante. Elle est empruntée à la
  distance de transition au contrôle LSO du **Case III** (NAVAIR 00-80T-104 §6.6.3.1), pas à un
  seuil Case I ; NAVAIR 00-80T-105 §6.2.4.2/6.2.4.3 définit le début du groove Case I comme un
  événement ("roll wings level on centerline with a centered ball"), jamais une distance/altitude
  fixe. `is_rolled_out()` (`src/track.rs`) ajoute donc deux proxies observables de cet événement,
  vérifiés sur `GROOVE_ROLLOUT_MIN_CONSECUTIVE_SAMPLES = 2` échantillons consécutifs une fois déjà
  dans la boîte : roulis quasi nul (`GROOVE_ROLLOUT_MAX_BANK_DEG = 15°`) et route sol déjà pointée
  dans l'axe du groove (`GROOVE_ROLLOUT_MAX_TRACK_ANGLE_DEG = 15°`, calculée sur la fenêtre
  `gate_samples` déjà bufferisée pour les gates, sans nouveau champ de télémétrie). Seuils
  `PROJECT-DERIVED`, non chiffrés par NATOPS, jamais revalidés en live. **CATOBAR uniquement** :
  V/STOL (Tarawa AV-8B) garde la boîte seule, son profil d'approche (hover/cross/VL, voir
  VSTOL.md) n'ayant pas de virage final CATOBAR à distinguer d'un survol transitoire de la boîte.
  Cette même géométrie de boîte reste, comme avant, non revue pour un éventuel Case II/III (non
  modélisé par ce projet — voir README.md, "Case I pattern") : n'étendre le raisonnement ci-dessus
  à Case II/III ou à un futur Case I V/STOL/LHA sans le revalider séparément contre leur propre
  doctrine.
- Franchissement du seuil de pont (`crossed_deck_threshold`, distingue `Bolter` de `WO?`) : ne se
  déclenche que si l'avion est proche du niveau du pont au moment du franchissement
  (`DECK_CROSSING_ALT_CAP_FT = 50 ft`, relatif au pont, crosse comprise) — sinon `WaveoffUnknown`.
  Corrige un bug confirmé live où une remise de gaz haute (~460 ft au franchissement) était classée
  `Bolter` ; voir [tasking-roadmap.md](tasking-roadmap.md).
- Touch-and-go (CQ, crosse relevée volontairement) vs `Bolter` (`Track::calibrated_hook_state`,
  `src/track.rs`) : distingués par la position de crosse lue depuis DCS
  (`UnitService.GetDrawArgumentValue`, `AirplaneInfo::hook_draw_argument`), jamais déduits du
  comportement seul. Index par type : F/A-18C et VNAO T-45 = 25, F-14A/F-14B/F-14B(U) = 1305
  (`F14_HOOK_DRAW_ARGUMENT`, `src/data.rs`) ; AV-8B = aucun (pas de crosse pour ce workflow). Seuils
  d'interprétation `<= 0.2` = up, `>= 0.8` = down, avec stabilité exigée (3 échantillons/0,4 s pour
  "up", 2/0,2 s pour "down" — barre plus haute pour "up" car c'est cette conclusion qui transforme
  le verdict par défaut `Bolter` en `TouchAndGo`), uniquement sur des échantillons pris dans le
  dernier 1/4 NM et avant l'enregistrement du toucher. **Seule la polarité 0,2/0,8 du F/A-18C a été
  confirmée empiriquement** (`HookObservation::polarity = "fa18c_zero_up_one_down_test_corpus"`) ;
  celle du T-45/F-14 est une extension non vérifiée de la même convention, fournie par
  l'utilisateur avec l'index mais pas la confirmation de polarité
  (`"assumed_zero_up_one_down_pending_live_validation"`). Sans index connu pour un type (ou lecture
  ambiguë/instable), le résultat reste `Unknown` → `Bolter` par défaut, jamais `TouchAndGo` inventé.
- Grading v4 (`project-derived-v4`) : en plus des trois gates ponctuelles, la trajectoire continue
  du groove au touchdown (`trajectory_deviations`) peut dégrader — jamais améliorer — l'amplitude
  retenue, sous réserve d'un garde de persistance (une seule frame aberrante isolée ne compte plus,
  il faut au moins 2 échantillons consécutifs au-dessus du seuil ; jamais appliqué au Cut ni à la
  pondération de fin d'approche, qui restent sensibles à un seul échantillon) ; un facteur de
  tendance plafonne une passe encore en train de s'aggraver dans les 4 dernières secondes à `(OK)`
  au lieu de `OK` ; une détection de surcorrection (NATOPS `OC`) plafonne de la même façon une passe
  montrant au moins 2 inversions de direction (écart net proche de zéro mais oscillation réelle,
  invisible au facteur de tendance seul) ; une pondération temporelle plafonne à `--` un écart
  modéré situé dans les 150 derniers mètres avant la coupe ; un Cut dédié (seuils
  `PROJECT-DERIVED`, **non chiffrés par NATOPS** — les deux NATOPS de référence ne codifient
  `TMRD`/`W`/`TMA`/`DLW`/`DRW` que comme codes de commentaire qualitatifs, jamais un nombre)
  sanctionne un sink rate (`SINK_RATE_CUT_MPS = 8.0 m/s`, environ le double du régime nominal de
  poser CATOBAR sans flare ~600-800 ft/min) ou une gîte (`BANK_ANGLE_CUT_DEG = 30°`, le double du
  seuil de roll-out `GROOVE_ROLLOUT_MAX_BANK_DEG`) soutenus (au moins 3 échantillons consécutifs,
  garde renforcée par rapport aux 2 échantillons habituels) à l'intérieur du 1/4 NM — même zone que
  le Cut GS. `trajectory_deviations` porte aussi `alt_m`/`bank_deg`/`sink_rate_mps` en contexte sur
  le reste de leur amplitude/tendance (non notée en dehors de ce Cut). Un échantillon n'est plus
  poussé sous
  `TRAJECTORY_MIN_DISTANCE_M` (3 m, `src/track.rs`) : `gs_deviation_deg`/`lineup_deg` sont
  `atan2(écart_m, x)`, et sous ce plancher un flare réaliste de quelques décimètres produisait un
  angle de plusieurs dizaines de degrés sans signification géométrique (bug confirmé live le 5
  septembre 2026, corrigé le même jour). Au-dessus de ce plancher mais en dessous de
  `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` (75 m, ajouté le 5 septembre 2026 suite à un second test live
  le même jour), le même effet existait sous une forme plus modérée mais tout aussi trompeuse : un
  écart quasi constant de quelques décimètres (confirmé présent dès 50 m sur les cinq appontages du
  second test, appontages propres comme Cut) grossissait mécaniquement jusqu'à des dizaines de
  degrés dans les derniers mètres, faisant retomber à `NoGrade` deux passes par ailleurs
  irréprochables — dont une notée `_OK_` par DCS lui-même. Corrigé en substituant cette distance de
  référence fixe à `x` dans le calcul d'angle une fois `x` sous ce seuil
  (`trajectory_deviation_angles_deg`, partagée par `Track::next` et `replay_gate_and_trajectory`) :
  un écart réel et important continue de dégrader la note comme avant, seul l'arrondi normal et
  attendu près du pont n'explose plus artificiellement. Voir
  [tasking-roadmap.md](tasking-roadmap.md) pour le détail des deux bugs et
  [docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md), "Near-touchdown geometry", pour la
  spécification complète.

CATOBAR conserve les règles projet existantes (`OK`, `(OK)`, `--`, `C`, `B`, `WO?`, `NC`), plus
désormais `_OK_` automatique (5 septembre 2026, `is_amplitude_perfect`/`grade_from_gates`,
`src/grading.rs`) : uniquement depuis une passe déjà `Ok` par toutes les règles ci-dessus, si en
plus (1) chaque porte et chaque échantillon continu reste dans `OK_PERFECT_GS_HIGH_DEG`/
`OK_PERFECT_GS_LOW_DEG` (+0,4°/-0,3°) et `OK_PERFECT_LU_ABS_DEG` (0,5°) — gardes sans pardon sur les
portes (preuves déjà validées bracket/skew), avec le même pardon anti-bruit qu'ailleurs
(`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`, 2) sur la trajectoire continue — et (2) le temps de groove
(`groove_time_secs`) est connu et tombe dans `15,0..=18,0` s (`OK_PERFECT_GROOVE_TIME_MIN_S`/
`_MAX_S`). Seuil de temps `OFFICIAL` (NAVAIR 00-80T-105 §6.2.4.3, "a 15 - 18 second groove"),
appliqué identiquement à tous les CATOBAR (F-14/F-18/T-45) malgré la pente différente du T-45,
faute de référence de vitesse d'approche par type — limite connue, non résolue. Bande d'amplitude
`PROJECT-DERIVED`, empruntée telle quelle au mod open-source MOOSE `Ops.Airboss`
(`AIRBOSS.GLE`/`AIRBOSS.LUE` `_max`/`_min`), même filiation historique que `GS_SLIGHT_*`/
`LU_SLIGHT`. Jamais lié au brin (l'ancien couplage MOOSE "brin 3 + 15-18,99 s" reste désactivé, aucun
NATOPS ne relie brin et note) ; un touch-and-go plafonne systématiquement un tier sous
`grade_from_gates`, jamais `_OK_`. `lso.exe cadence-ab` ne rejoue pas la détection de toucher, donc
`groove_time_secs` y vaut toujours `None` : `_OK_` n'apparaît jamais dans une note rejouée, seul
l'usage live peut l'émettre. Non revalidé en mission live. Détail complet dans
[docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md), "Automatic `_OK_`".

Contact sans arrest confirmé : `UnconfirmedArrest`, aucun point. Le
câble DCS/LQM confirme seulement une valeur strictement comprise entre 1 et 4 ; 0, >4, overflow,
négatif ou format mal formé sont rejetés. Câble Rust et DCS restent séparés avec
provenance/divergence/confiance ; les surfaces pilote (Discord, PNG, SQLite/board) n'affichent
jamais l'estimation Rust si elle diverge du câble DCS/LQM affiché — seul le JSON complet garde les
deux valeurs pour diagnostic. `cable_estimated`/`wire_estimated` (grading) est désormais toujours
dérivé de `wire_estimation` (diagnostic JSON), calculé une seule fois dans `Track::finish()` contre
l'historique complet des franchissements de brin, plutôt que capturé séparément au moment du
toucher : les deux champs pouvaient diverger sur un même rapport selon que la corrélation
événementielle du `Land` tombait avant ou après le tick positionnel ayant ajouté le franchissement
correspondant (bug confirmé live le 5 septembre 2026, corrigé le même jour).

V/STOL reste AV-8B/Tarawa, spot intentionnel 7.5, formule locale expérimentale décrite dans
[VSTOL.md](VSTOL.md). Intended spot, nearest active spot et distance sont séparés. Jamais de note
favorable si incomplet.

AoA dans `datums`/`pattern_datums` est corrigé du vent une fois une référence de vent établie
(deux appels `AtmosphereService.GetWind` à l'entrée du groove, interpolés par altitude), sinon
retombe sur l'approximation brute (jamais une valeur fabriquée) ; `wind_reference_established`
enregistre lequel des deux cas s'est produit. AoA reste affiché/loggé uniquement, jamais noté.

## Événements et complétude

Une panne ou fermeture propre de `StreamEvents` :

- ajoute `event_stream_unavailable` comme diagnostic secondaire ;
- conserve gates et métriques positionnelles ;
- n'appelle jamais `mark_telemetry_gap` ;
- laisse la collecte de positions continuer ;
- expose `event_correlation.stream_status`, détail, preuve antérieure et `outcome_confirmed` ;
- rend confiance/availability insuffisantes si l'outcome dépendant des événements n'est pas
  confirmé ;
- ne retire pas un touchdown/LQM déjà confirmé ;
- ne bloque pas un outcome confirmé indépendamment par les positions, par exemple un bolter.

Les overflows hook/event sont diagnostiques. Seule la perte du buffer positions peut produire
`BufferLimit`.

## Respawns et isolation

Le registre stocke session, génération, IDs et noms. Un avion ou carrier de même nom avec nouvel ID
annule toutes les tâches de l'ancienne incarnation dans la même session/génération. Les autres
unités et générations restent isolées. Les guards prioritaires sont relâchés à l'abort. La
suspension des détecteurs reste limitée au même avion, donc une autre recovery simultanée peut
être découverte.

## Mode `--positions-only`

Conserve seulement le collecteur et le JSON diagnostic :

- aucune lecture/validation de `--discord-users`, webhook forcé à `None` ;
- pas de SQLite, dashboard, session board ou Discord ;
- pas de hook, canal hook ou client legacy ;
- pas de stream d'événements recovery ni métrique de stream correspondante ;
- pas de writer/metadata/unit RPC ACMI ;
- pas de World/theatre ni mission-time output-only ;
- pas de PNG/rendu ;
- JSON, provenance, cadence, gaps, source age, skew et latences positions conservés.

Le stream superviseur Birth/session reste nécessaire à la découverte et l'isolation des unités.

## Persistance et atomicité

`recovery_id = s<session>-g<generation>-p<plane>-c<carrier>-t<dcs_ms>`.

- Aucun `try_exists + rename`.
- Écriture dans un temporaire du même dossier, flush + `sync_all`, puis `hard_link(temp,
  destination)`.
- Création atomique sans remplacement sur Windows et Unix.
- Temporaires et répertoires de rendu nettoyés.
- Claim process-scoped `(out_dir, recovery_id)` contre deux noms concurrents.
- Le JSON identifie le producteur gagnant ; seul lui poursuit ACMI, SQLite, rendu, session log et
  Discord.
- Un artefact existant n'est jamais remplacé.

SQLite : migrations additives 2–6 (`schema_migrations`), index unique partiel `recovery_id`,
`INSERT OR IGNORE`. Discord seulement pour une nouvelle ligne. UCID uniquement SQLite/API privée,
jamais JSON/PNG/ACMI/Discord/log public. Dashboard loopback `127.0.0.1`, sans OAuth/TLS, privé
phase 1.

## Provenance Git et baseline

`build.rs` injecte commit et dirty. Dirty = `git status --porcelain=v1 --untracked-files=no` :
modifications/suppressions/staging suivis participent ; fichiers non suivis et `target/` ne
participent pas. Tous les chemins suivis, index, HEAD et ref active déclenchent Cargo ; `build.rs`
et `build_support.rs` ont des triggers explicites. `tests/build_provenance.rs` teste la logique
déterministe.

`--baseline-manifest` refuse objet vide, clés inconnues, valeurs vides et SHA-256 mal formés. Les
erreurs affichent chemin, ligne, colonne et cause système.

## Diagnostics d'erreur

[src/error.rs](src/error.rs) préserve `source` et affiche message système IO, chemin
contextualisé, ligne/colonne JSON, détail SQLite, rendu, ACMI et Discord. Le point de terminaison
journalise display et chaîne debug. Les échecs SQLite/PNG/ACMI/Discord arrivent après
`Track::finish` et ne modifient jamais rétroactivement les preuves positionnelles.

## Contrats de données

JSON reste `schema_version: 3`, évolution additive : aucun ancien champ supprimé/renommé ; `cause`
reste l'alias primaire ; `causes` contient primaire/secondaires ; `event_correlation`,
`wind_heading_deg`/`wind_speed_mps`, `wind_reference_established` et `trajectory_deviations` sont
des ajouts récents ; `trajectory_deviations[].alt_m`/`bank_deg`/`sink_rate_mps` et
`datums[].roll_deg` sont des ajouts additifs plus récents encore (contexte sur leur
amplitude/tendance générale, mais `bank_deg`/`sink_rate_mps` alimentent chacun le Cut dédié
sink-rate/bank — voir "Gates, outcomes et câble") ; diagnostics possibles
`event_stream_unavailable` ; `grading_availability` peut valoir
`unavailable_event_outcome` ; `groove_time_secs` (ajout du 5 septembre 2026) sérialise désormais
dans le JSON la donnée déjà utilisée pour `_OK_` automatique, auparavant calculée mais visible
seulement dans l'embed Discord — un rapport live sans Discord configuré ne permettait alors aucune
vérification a posteriori de l'éligibilité `_OK_`. Détail exhaustif dans
[docs/DATA_CONTRACTS.md](docs/DATA_CONTRACTS.md).

SQLite utilise le vocabulaire snake_case du JSON. L'absence d'un nouveau champ signifie
legacy/unknown, jamais favorable.

## CI et sécurité

`.github/workflows/ci.yml` exécute build/test avec `--locked`, Clippy `--locked --all-targets --
-D warnings`, rustfmt, installation épinglée de `cargo-audit 0.21.2 --locked`, puis `cargo audit`.
Ne jamais modifier silencieusement `.cargo/audit.toml`.

## Fichiers de contexte associés

- [primer.md](primer.md) : explication human-first, vulgarisée, du fonctionnement complet du
  module (DCS / DCS-gRPC / DCS-gRPC-lso, les 8 étapes d'un appontage noté).
- [tasking-roadmap.md](tasking-roadmap.md) : roadmap technique, décisions ouvertes, bugs connus et
  historique synthétique des sessions de développement/tests.
- [README.md](README.md), [CHANGES.md](CHANGES.md) : usage et changelog utilisateur.
- [docs/DATA_CONTRACTS.md](docs/DATA_CONTRACTS.md), [docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md),
  [docs/RELIABILITY_ARCHITECTURE.md](docs/RELIABILITY_ARCHITECTURE.md),
  [docs/LSO_ANALYSIS.md](docs/LSO_ANALYSIS.md) : spécifications techniques actuelles, tenues à jour
  indépendamment de ce document.
- [docs/NOTATION-LOGIQUE.md](docs/NOTATION-LOGIQUE.md) : résumé pédagogique, sans jargon de
  programmation, du raisonnement de notation CATOBAR Case I — un extrait ciblé de
  [docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md) (qui fait foi en cas de différence), pour
  qui veut comprendre la logique de note sans lire la spécification technique complète.
- [VSTOL.md](VSTOL.md) : spécification V/STOL AV-8B/Tarawa.

Les anciens documents `.ignore/*.md` et `.agents/agents.md` qui ont servi de brouillon à ce
document ont été supprimés lors du ménage documentaire du 4 septembre 2026 pour éviter toute
divergence future ; leur contenu pertinent est consolidé ici et dans
[tasking-roadmap.md](tasking-roadmap.md).
