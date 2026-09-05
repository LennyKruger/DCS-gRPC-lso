# AGENTS.md — Contexte machine-first pour DCS-gRPC-lso

> Document de continuité, à tenir à jour à chaque changement significatif de code ou de contrat.
> Dépôt `E:\DCS stuffs\Initiative ESG\DCS-gRPC-lso`, branche `feature/refonte-v3-lua-buffer`.
> Dernier état vérifié : HEAD `5f52e13` ("Dernière modification avant test humans 05/09/2026"),
> working tree propre à l'exception de ce ménage documentaire. Ce commit contient, entre autres, le
> raffinement CATOBAR de l'entrée en groove par confirmation de roulis/route, le correctif de la
> désynchronisation `wire_estimated`/`wire_estimation`, le plancher de distance sous lequel
> `trajectory_deviations` n'est plus poussé (évite l'explosion `atan2` près du toucher) puis son
> complément `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M`, le nouveau Cut sink-rate/bank-angle soutenu près du
> pont, `_OK_` automatique (amplitude MOOSE-inspirée + temps de groove NATOPS 15-18 s), la
> calibration de la position de crosse (T&G vs Bolter) étendue au T-45 et aux F-14 via leurs propres
> index de draw argument, et `groove_time_secs` désormais exposé dans le JSON. Détail synthétique
> daté dans [CHANGES.md](CHANGES.md) (section `Unreleased`) ; ce qui reste à revalider en mission
> live est dans [tasking-roadmap.md](tasking-roadmap.md). Crate `lso` 0.2.0, Rust 2021 ; les
> changements postérieurs au tag `0.2.0` sont sous `Unreleased` dans [CHANGES.md](CHANGES.md).

Pour un résumé humain, vulgarisé, du fonctionnement du module : voir [primer.md](primer.md).
Pour la roadmap, les décisions ouvertes et les bugs connus non résolus : voir
[tasking-roadmap.md](tasking-roadmap.md). Pour l'historique synthétique de toutes les versions : voir
[CHANGES.md](CHANGES.md).

## Règles de maintenance des documents markdown racine

Ce dépôt maintient quatre documents markdown à la racine selon des règles strictes, à appliquer
automatiquement par le modèle à la fin de toute session de travail ayant modifié le code, le
contrat de données ou la conception du projet de façon significative — sans attendre une demande
explicite de l'utilisateur :

- **[AGENTS.md](AGENTS.md)** (ce fichier) : machine-first, technique, exhaustif sur l'état courant.
  Reconstruit/édité pour ne refléter que l'état factuel actuel du code et des contrats — jamais de
  trace d'anciennes versions, de formulation "auparavant X, maintenant Y" une fois la transition
  terminée, ni de journal de session. En cas de changement significatif : mettre à jour ce fichier
  en fin de session, supprimer toute information devenue obsolète plutôt que l'annoter comme
  périmée.
- **[tasking-roadmap.md](tasking-roadmap.md)** : roadmap technique et humaine — idées, choix
  ouverts, arbitrages à faire, bugs confirmés mais non corrigés, corrections faites mais non encore
  revalidées en mission live. Dès qu'un chantier qui y était listé est terminé **et** revalidé (ou
  dès qu'un point est explicitement tranché sans besoin de revalidation), son entrée est supprimée
  d'ici ; le fait qu'il a été fait, et l'essentiel technique du "pourquoi", migre dans
  [CHANGES.md](CHANGES.md) (et dans AGENTS.md si ça touche l'état durable du système). Ne doit
  contenir, à tout instant, que des points **encore en suspens** — jamais un historique de ce qui a
  déjà été résolu.
- **[primer.md](primer.md)** : human-first, vulgarisé pour un public non-développeur, présente le
  projet dans son ensemble et la logique de notation en détail mais sans jargon de programmation.
  Mis à jour après tout changement de comportement observable par un pilote/LSO humain (nouvelle
  règle de notation, nouveau champ de rapport visible, correctif changeant un verdict). Purgé des
  informations obsolètes à chaque mise à jour : un bug corrigé n'y reste documenté que si comprendre
  qu'il a existé aide à comprendre la règle actuelle (sinon, le retirer) ; jamais deux versions
  contradictoires d'une même règle.
- **[CHANGES.md](CHANGES.md)** : technique, historique, format changelog (Added/Changed/Fixed/
  Security, section `Unreleased` depuis le tag `0.2.0`). Mis à jour à chaque fois que les documents
  ci-dessus perdent une information devenue obsolète suite à un changement significatif : la version
  synthétique de ce qui a été fait (quoi, pourquoi en une ligne, quel fichier de code) y est ajoutée
  au même moment, jamais reconstruite plus tard de mémoire. Ne pas y remettre le détail narratif de
  session (dates de test, discussions de conception) qui appartient à l'historique Git lui-même — un
  `git log`/`git show` sur le commit correspondant fait foi pour ce niveau de détail.

Ordre d'application typique en fin de session : coder + tester -> mettre à jour AGENTS.md avec le
nouvel état factuel -> ajouter l'entrée correspondante dans CHANGES.md -> retirer de
tasking-roadmap.md ce qui est désormais résolu (ou ajouter ce qui reste ouvert) -> mettre à jour
primer.md si le comportement observable par un utilisateur a changé.

Aucun document technique sous `docs/` ne subsiste indépendamment de ce document : le déploiement/
rollback, la spécification de notation détaillée et les contrats de données sont documentés
directement dans ce fichier plus bas. La vulgarisation sans jargon de la logique de notation vit
dans [primer.md](primer.md).

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

Dernière validation locale complète au commit `5f52e13` (5 septembre 2026, voir en tête de document
pour le contenu de ce commit) :

- `cargo test --locked --no-fail-fast` : **196 réussis, 0 échec** (194 tests du binaire + 2 tests
  de provenance de build) ;
- `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` propres ;
- Working tree propre à ce commit (aucune modification de code en attente).

Un test live CVN-72 (4×F-14 + 4×F-18 IA) a été mené le 5 septembre 2026 au soir sur ce commit et a
confirmé l'absence de régression sur les correctifs déjà en place à ce moment-là, mais chacun des
changements listés en tête de document reste **non revalidé sur un enregistrement live postérieur à
son propre correctif** — voir [tasking-roadmap.md](tasking-roadmap.md), "Décisions encore
ouvertes", pour le détail de ce qui reste à confirmer en mission et sur quelles preuves live
passées.

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
(voir "Gates, outcomes et câble" plus bas, qui fait foi pour la spécification exacte). Ne pas
modifier les règles CATOBAR/V/STOL sans demande dédiée.

## DCS-gRPC et dépendances

- Stubs : **dépendance de chemin local** vers `../DCS-gRPC/stubs` (`Cargo.toml`
  `[dependencies.stubs] path = "../DCS-gRPC/stubs"`), alignée sur le checkout frère
  `E:\DCS stuffs\Initiative ESG\DCS-gRPC`, workspace `dcs-grpc v0.10.0`. Ce n'est **pas** un
  tag/rev Git figé : le fork contient `RecoveryService` (start/read/stop telemetry), et LSO en
  dépend directement pour la source bufferisée par défaut. Avant tout packaging release, remplacer
  ce chemin local par un pin Git immuable et revu.
- `tonic = 0.13`, Axum direct 0.8 (aligné sur la ligne de Tonic). Contrainte durable : les clients
  générés par les stubs sont paramétrés par les types de transport de **leur propre** version de
  `tonic` (`tonic::transport::Channel` passé à `MissionServiceClient`/`UnitServiceClient`/
  `WorldServiceClient`, etc.) — la version `tonic` directe de LSO doit donc rester alignée
  major/minor avec celle déclarée par `dcs-grpc-stubs`, jamais choisie indépendamment.
- Le champ protobuf `dcs.common.v0.Unit.type` est optionnel (`Option<String>`) : une unité DCS sans
  type n'est jamais retenue comme candidate recovery (`check_candidate`, `src/commands/run.rs`,
  via `unit.r#type.as_deref().and_then(AirplaneInfo::by_type)` — `None` ne matche jamais), aussi
  bien à la découverte initiale qu'aux événements `Birth` ultérieurs. Nécessaire : sans type, LSO ne
  peut pas choisir en sécurité l'offset de crosse, les positions de brins, le glideslope ou les
  dimensions du porte-avions.
- URI par défaut `http://127.0.0.1:50051`, deadline/connect timeout 2 s, retry exponentiel sans
  limite totale, intervalle max 30 s.
- Compatibilité serveur : `dcs_grpc_compatibility` est calculé à l'exécution contre le serveur
  réellement connecté ; classification `compatible_same_api_line` avec avertissement pour une
  autre version mineure de la même ligne, `incompatible` pour une autre ligne. Aucune compatibilité
  fonctionnelle au-delà de cette classification n'a été validée live avec la version actuelle des
  stubs.
- Commit serveur ciblé (non publié) : `c6fb3f7737f48c82601866f696d7df66ac727414` — c'est la révision
  que le chemin local `../DCS-gRPC/stubs` ci-dessus doit refléter côté checkout frère.

**Procédure pour un futur repin du fork** (quand le chemin local sera remplacé par un tag/rev Git
immuable, ou lors d'une mise à jour ultérieure de ce pin) :

1. Revoir l'historique du fork, le tag/release visé et le SHA de commit exact.
2. Comparer `Cargo.toml`, `stubs/Cargo.toml`, les changements protobuf, les dépendances de build et
   le changelog du dépôt serveur.
3. Ne changer que la valeur `tag`/`rev` de `[dependencies.stubs]`, sauf si le fork change aussi sa
   version `tonic` requise (voir contrainte ci-dessus) — auquel cas aligner la version `tonic`
   directe de LSO en même temps.
4. `cargo update -p dcs-grpc-stubs` pour régénérer le lockfile.
5. `cargo test --locked --no-fail-fast`, puis `cargo audit`.
6. Vérifier que `Cargo.lock` enregistre bien le SHA de commit complet visé.
7. Mettre à jour les références de version/commit dans [README.md](README.md) et ce document.
8. Test de fumée live contre un DCS World réel avant tout déploiement du nouveau binaire LSO —
   aucune promotion sur la seule base des tests unitaires/`cargo audit`.

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
- [src/tasks/detect_recovery_attempt.rs](src/tasks/detect_recovery_attempt.rs) : détecteur par
  paire, vérifié toutes les 2 s. Enveloppe de repérage d'un début d'approche (`is_recovery_attempt`)
  : altitude avion `<= 1100 ft`, distance au porte-avions `<= 3.5 NM` et `> 200 m` (exclut un avion
  déjà posé/en train de décoller). Volontairement sans vérification de cap pointé vers le
  porte-avions ni d'hémisphère arrière : pendant le break, l'avion est abeam avec le nez perpendiculaire
  à la BRC, et le circuit "overhead" place l'avion devant le porte-avions (initial/break) — ces deux
  vérifications auraient exclu de vraies approches légitimes.
- [src/telemetry.rs](src/telemetry.rs) : politique de fraîcheur monotone, skew, extrapolation et
  reset après coupure — voir "Contrat de télémétrie" plus bas pour les seuils exacts.
- [src/grading.rs](src/grading.rs) : score projet CATOBAR et modèle V/STOL expérimental — voir
  "Gates, outcomes et câble" plus bas.
- [src/db.rs](src/db.rs) : migrations additives et persistance privée idempotente — voir
  "Persistance et atomicité" plus bas.
- [src/metrics.rs](src/metrics.rs) : instrumentation RPC/stream/queue/IO/rendu — voir
  "Observabilité runtime" plus bas.
- [src/web.rs](src/web.rs) : dashboard privé loopback-only.
- [src/draw.rs](src/draw.rs) : rendu PNG (approche + pattern), déporté en `spawn_blocking`.

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
- gap/source age >1 000 ms **à l'intérieur d'une gate/du groove noté** : `TelemetryGap`, télémétrie
  de notation incomplète, aucun point (`telemetry_gap_only_invalidates_the_scored_segment`,
  `src/track.rs`) ;
- le même gap >1 000 ms **hors du segment noté** (pattern/break avant le groove) reste un
  diagnostic conservé, sans invalider la note à lui seul ;
- watchdog sans progression source : 2 s ;
- reset de l'aligneur après erreur ;
- timestamps DCS, réception Unix et horloge monotone distincts.

Aucune baseline live récente n'est revalidée dans ce document — voir
[tasking-roadmap.md](tasking-roadmap.md) pour les dernières mesures live datées et leurs limites.
Des rotations répétées du watchdog indiquent un producteur silencieux ou un canal gRPC dégradé, pas
une raison d'augmenter ce délai de 2 s sans mesure préalable.

## Gates, outcomes et câble

Sources `OFFICIAL` utilisées pour le vocabulaire/les symboles (jamais pour les formules/seuils
géométriques ni le bonus V/STOL A/B/C/D, tous `PROJECT-DERIVED`) : NAVAIR 00-80T-104 (1 mai 2009)
§6.3.2 (terminologie touch-and-go), §6.6.4 (contexte foul-deck waveoff) et §11.4.1 (symboles de
note) ; NAVAIR 00-80T-105 §6.2.4.2/6.2.4.3 (groove Case I) ; NAVAIR 00-80T-111 (15 décembre 2004)
chapitre 23 et fiches A-5/A-9 (phases V/STOL, évaluation humaine hover/cross/VL/puissance/
assiette/spot/cap relatif).

Copies PDF locales de ces manuels, déposées sous `docs/` : `docs/LSO-NATOPS-MAY09.pdf` (NAVAIR
00-80T-104), `docs/CV-NATOPS-JUL09.pdf` (NAVAIR 00-80T-105), `docs/AV8-CASE-I-II-III.pdf` (extrait
chapitre 6 de NAVAIR 00-80T-111, procédures de recovery Case I/II/III V/STOL), `docs/AV8-CVN-SPOT.pdf`
(diagramme de référence des spots de pont AV-8B — image, sans texte extractible). **Ces quatre PDF
sont des documents doctrinaux de référence, pas une spécification du projet** : ils ne servent qu'à
vérifier, ponctuellement et manuellement, qu'une règle `OFFICIAL` citée dans ce fichier correspond
bien à la doctrine réelle — jamais à en extraire de nouveaux seuils numériques pour le grading sans
une décision explicite de l'utilisateur, et jamais à les traiter comme une source de vérité sur
l'état du code (voir "Règles de vérité" plus haut).

- Gates : 3/4 NM 1 389 m, 1/2 926 m, 1/4 463 m.
- États : `Missing`, `Late`, `Invalid`, `Valid`.
- Validité : deux samples inbound encadrants, temps croissant, bracket <=300 ms, skew <=300 ms,
  phase/altitude admissibles.
- Trois gates valides et ordonnées sont obligatoires pour une note favorable.
- Démarrage à l'intérieur : `Late`, jamais de donnée inventée.
- Formule au seuil `x` : `ideal_alt = base_alt + x * tan(pente_avion)` ;
  `gs_deg = atan2(observed_alt - ideal_alt, x)` ; `lineup = atan2(écart_latéral, x)` (voir
  "Near-touchdown geometry" plus bas pour le cas `x` proche de zéro).
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
- Déclencheur commun touch-and-go/Bolter : la distance au point de toucher atteint un minimum puis
  regrossit de plus de 150 m (`src/track.rs`) — l'avion a touché puis est reparti sans s'arrêter.
  Seule la position de crosse à cet instant distingue ensuite les deux issues.
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
  L'échantillonnage de crosse est un sampler indépendant, hors du chemin critique 10-20 Hz de
  position : défaut 4 Hz / timeout 300 ms, configurable dans 2-4 Hz / 250-300 ms
  (`--hook-sampling-hz`, `--hook-timeout-ms`) ; `--legacy-inline-hook-sampling` restaure l'ancien
  échantillonnage bloquant inline comme rollback A/B (voir "Observabilité runtime").
- Grading v4 (`project-derived-v4`) : en plus des trois gates ponctuelles, la trajectoire continue
  du groove au touchdown (`trajectory_deviations`) peut dégrader — jamais améliorer — l'amplitude
  retenue, sous réserve d'un garde de persistance (une seule frame aberrante isolée ne compte plus,
  il faut au moins 2 échantillons consécutifs au-dessus du seuil ; jamais appliqué au Cut ni à la
  pondération de fin d'approche, qui restent sensibles à un seul échantillon) ; un facteur de
  tendance (`TREND_WINDOW_S = 4 s`) plafonne à `(OK)` au lieu de `OK` une passe dont la pente
  GS/lineup s'aggrave encore d'au moins `TREND_WORSENING_DEG_PER_S = 0,075°/s` sur cette fenêtre ;
  une détection de surcorrection (NATOPS `OC`, même fenêtre de 4 s) plafonne de la même façon une
  passe montrant au moins `OSCILLATION_MIN_REVERSALS = 2` inversions de direction d'au moins
  `OSCILLATION_MIN_SWING_DEG = 0,3°` chacune (écart net proche de zéro mais oscillation réelle,
  invisible au facteur de tendance seul) ; une pondération temporelle (`LATE_WINDOW_DISTANCE_M =
  150 m`) plafonne à `--` un écart franchissant `LATE_WINDOW_GS_DEG = 0,8°` ou `LATE_WINDOW_LU_DEG =
  1,5°` (entre les seuils généraux `*_SLIGHT`/`*_SIGNIFICANT`) dans les 150 derniers mètres avant la
  coupe, jamais appliqué à un `NoGrade`/`Cut` déjà acquis ni au Cut lui-même ; un Cut dédié (seuils
  `PROJECT-DERIVED`, **non chiffrés par NATOPS** — les deux NATOPS de référence ne codifient
  `TMRD`/`W`/`TMA`/`DLW`/`DRW` que comme codes de commentaire qualitatifs, jamais un nombre)
  sanctionne un sink rate (`SINK_RATE_CUT_MPS = 8.0 m/s`, environ le double du régime nominal de
  poser CATOBAR sans flare ~600-800 ft/min) ou une gîte (`BANK_ANGLE_CUT_DEG = 30°`, le double du
  seuil de roll-out `GROOVE_ROLLOUT_MAX_BANK_DEG`) soutenus (au moins 3 échantillons consécutifs,
  garde renforcée par rapport aux 2 échantillons habituels) à l'intérieur du 1/4 NM — même zone que
  le Cut GS. `sink_rate_mps` = `d(altitude)/dt` entre deux échantillons continus consécutifs
  (positif = descend, `0.0` pour le premier échantillon d'une passe). `trajectory_deviations` porte
  aussi `alt_m`/`bank_deg`/`sink_rate_mps` en contexte sur
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
  [tasking-roadmap.md](tasking-roadmap.md) pour le détail des deux bugs.

**Table de note CATOBAR** (`project-derived-v4`, `PROJECT-DERIVED` sauf mention contraire ;
`abs(GS)`/`abs(LU)` = pire valeur sur les trois gates **et** la trajectoire continue) :

| Résultat | Règle | Points |
|---|---|---:|
| `_OK_` | `OK` ci-dessous resserré à `abs(GS) <= 0,4/0,3°` (haut/bas) et `abs(LU) <= 0,5°` partout, **et** temps de groove 15-18 s ; jamais pour un touch-and-go | 5.0 |
| `OK` | trois gates valides ; `abs(GS) < 0,5°`, `abs(LU) < 1,0°` | 4.0 |
| `(OK)` | pas d'écart significatif ; `abs(GS) >= 0,5°` ou `abs(LU) >= 1,0°` | 3.0 |
| `--` | `abs(GS) >= 1,0°` ou `abs(LU) >= 2,0°` | 2.0 |
| `C` | GS strictement sous `-2,5°` à la gate 1/4 NM, ou n'importe où dans la trajectoire continue à 463 m ou en dessous ; ou sink rate soutenu (>=3 échantillons) `>= 8,0 m/s` ou gîte `>= 30°` à l'intérieur de 463 m | 0.0 |
| `B` | bolter confirmé et trois gates valides | 2.5 |
| `WO?` | remise de gaz/go-around neutre, initiateur inconnu | aucun |
| `NC` | télémétrie insuffisante/invalide ou trap non confirmé | aucun |

**Ce que `NC` recouvre réellement** — `NC` est un seul symbole d'affichage, jamais une seule raison
interne ; `cause`/`causes` (voir "Contrats de données") distinguent déjà :

| Valeur `cause` | Signification | Catégorie |
|---|---|---|
| `telemetry_gap` | gap dans le segment noté au-delà des limites de gap/extrapolation | Télémétrie trop dégradée pour mesurer |
| `invalid_telemetry` | un sample noté a échoué à la validation (skew, temps non croissant...) | Télémétrie trop dégradée pour mesurer |
| `position_buffer_limit` | buffer de position débordé/perdu dans le segment noté | Télémétrie trop dégradée pour mesurer |
| `insufficient_gates` | télémétrie correcte, mais moins de trois gates valides/ordonnées | Structurel — rien à noter |
| `unconfirmed_arrest` | contact observé mais aucun brin DCS/LQM ne confirme un arrêt | Preuve manquante, pas une mesure |
| `unknown` (`Grading::Unknown`) | avion suivi mais jamais devenu une approche notée | Pas un problème de télémétrie du tout |

`cause` est la valeur de plus haute priorité (`Completeness::priority`, `src/track.rs`) ;
`causes.secondary` liste le reste. Un consommateur qui a besoin de cette distinction lit déjà
`cause`/`causes`, jamais le symbole `NC` seul.

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
l'usage live peut l'émettre. Non revalidé en mission live.

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
favorable si incomplet. Le catalogue géométrique actif ne contient que le spot 7.5 calibré ; les
spots 7 et 8 sont des candidats explicites pour le futur, ni actifs ni notés tant qu'aucune
calibration live n'existe pour eux. La vitesse horizontale au premier contact est conservée (`first-contact
horizontal speed`) pour permettre à une future évidence de distinguer VL et RVL sans fabriquer de
seuil aujourd'hui. Un contact suivi d'un départ est normalisé en un go-around/touch-and-go neutre,
jamais en `Bolter` (concept CATOBAR non transposé tel quel). Des contacts dupliqués sont conservés
comme preuve de robustesse plutôt qu'écrasés. Aucune de ces observations ne prouve l'ordre ou la
fiabilité des événements DCS réels côté Tarawa — voir [tasking-roadmap.md](tasking-roadmap.md).

AoA dans `datums`/`pattern_datums` est corrigé du vent une fois une référence de vent établie
(deux appels `AtmosphereService.GetWind` à l'entrée du groove, interpolés par altitude), sinon
retombe sur l'approximation brute (jamais une valeur fabriquée) ; `wind_reference_established`
enregistre lequel des deux cas s'est produit. AoA reste affiché/loggé uniquement, jamais noté. Les
tables `aoa_rating` par type (`src/data.rs`) viennent de documentation publique : bracket indexeur
VRS pour le F/A-18C, manuel Heatblur pour le F-14 (conversion `degrees=((units/1.0989)-3.01)`),
DisplayElectronicsUnit décompilé pour le VNAO T-45 v1.0.2 — ces tables classent la valeur AoA déjà
calculée, elles ne la mesurent pas elles-mêmes.

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

## Observabilité runtime

Le mode live (`lso run`) journalise un instantané cumulatif toutes les 10 s (`src/metrics.rs`),
purement observationnel (jamais utilisé pour une décision métier) :

- appels RPC unary et RPC/s, avec latence moyenne/p50/p95/p99/max séparée par catégorie (transform
  avion, transform carrier, autres transforms, hook) et compteurs succès/erreurs/timeouts propres à
  chacune ;
- durée complète de la boucle de recovery et retard de tick Tokio (moyenne/p50/p95/p99/max) ;
- nombre de streams d'événements et de recoveries actifs ;
- `queue_high_watermark` de la file du superviseur (capacité 16) ;
- octets écrits par la publication atomique ACMI/JSON ;
- nombre de rendus PNG et temps de rendu moyen (rendu déporté en `spawn_blocking`, hors boucle
  d'échantillonnage 10-20 Hz).

Les rapports de track ajoutent par-dessus le gap/skew maximum, les compteurs invalid/warning/dropped
et la complétude des buffers bornés.

### Protocole de benchmark

Une optimisation n'est acceptée qu'après une baseline reproductible et seulement si la qualité de
la télémétrie ne régresse pas (voir "Invariants d'optimisation" ci-dessous). Le rejeu hors-ligne
valide la mécanique CPU/RAM/rendu/IO ; RPC, streams, skew et FPS/tick DCS nécessitent le corpus
serveur live.

**Baseline hors-ligne** : builder une fois, puis lancer au moins 20 exécutions propres de la même
fixture depuis un dossier de sortie vide, et relever médiane/p95/max :

```powershell
cargo build --release --locked
$samples = 1..20 | ForEach-Object {
  $process = Start-Process -FilePath .\target\release\lso.exe `
    -ArgumentList @('file','tests\recordings\wire_3_01_T45.zip.acmi') `
    -PassThru -NoNewWindow
  $process.WaitForExit()
  $process.Refresh()
  [pscustomobject]@{
    ExitCode = $process.ExitCode
    CPU_s = $process.TotalProcessorTime.TotalSeconds
    PeakWorkingSet_MB = $process.PeakWorkingSet64 / 1MB
  }
}
$samples | Measure-Object CPU_s,PeakWorkingSet_MB -Average -Minimum -Maximum
```

Consigner aussi octets d'entrée/sortie et temps mur, ainsi que le diff Git exact, la version Rust,
le profil de build, le modèle CPU, la RAM et l'OS avec chaque résultat.

**Matrice live** : rejouer des missions identiques en baseline et en candidat — (1) une recovery
Hornet/CVN, (2) une recovery AV-8B/Tarawa, (3) Hornet/CVN et AV-8B/Tarawa simultanés, (4) 40 joueurs
sur deux navires, (5) cas de stress à trois porte-avions, (6) délai gRPC délibéré, gaps 300 ms/1 s,
reconnect et rotation de mission. Capturer à la résolution de la seconde : CPU/working-set/private
bytes du process, les métriques runtime ci-dessus (RPC/s, nombre de streams), octets disque/s et
high-water marks de file/buffer, percentiles p50/p95/p99 de latence transform/skew/gap, durée de
rendu PNG, et FPS/tick de simulation DCS via l'outil serveur convenu. Au moins dix minutes de
régime stable par run, plus toutes les recoveries ; conserver logs bruts anonymisés et hash de
mission.

**A/B du hook indépendant** : rejouer deux fois la même mission (mêmes hash serveur/mission) :

```powershell
# Candidat : le hook ne peut jamais retarder les transforms
.\lso.exe -v run -o C:\LSO\ab-independent --no-acmi --hook-sampling-hz 4 --hook-timeout-ms 300

# Rollback/contrôle : ancien comportement bloquant
.\lso.exe -v run -o C:\LSO\ab-inline --no-acmi --legacy-inline-hook-sampling
```

Comparer percentiles RPC transform, percentiles de lag boucle/tick, fréquence d'échantillonnage
réelle, source age, gaps et gates valides. `--no-acmi` est volontaire : la disponibilité ACMI ne
fait pas partie de la notation nominale. Le mode indépendant n'est acceptable que s'il préserve ou
améliore la cadence positionnelle sans jamais transformer une observation de hook périmée/inconnue
en certitude.

**Déjà sûr par invariant, pas la peine de re-benchmarker** : la matrice de compatibilité stricte
(paires Cartésiennes invalides exclues) est une correctness requise, pas une réduction de cadence
issue d'un benchmark ; les buffers bornés empêchent une croissance mémoire illimitée ; l'écriture
atomique, l'insertion SQLite idempotente et le rendu en `spawn_blocking` protègent la correction
sous passes concurrentes. Tout travail supplémentaire de cache de transform, déduplication RPC ou
allocation reste conditionné à une mesure live — voir les invariants ci-dessous.

**Aucune preuve n'existe encore** pour la charge 40 joueurs/trois porte-avions ni pour l'impact sur
le FPS/tick de simulation DCS : ne jamais l'affirmer sans une mesure live nouvelle (voir "Règles de
vérité").

### Protocole de capture de preuve live

Pour qu'une session live compte comme preuve (et pas seulement comme test manuel) : un dossier par
test, nommé heure UTC + ID de scénario, contenant build/versions DCS et hash de mission, SHA-256 du
DLL déployé et de chaque fichier Lua DCS-gRPC, log DCS, log DCS-gRPC, trace LSO, JSON, ACMI optionnel
et log de l'observateur d'événements indépendant, début/fin UTC synchronisés plus session/génération
DCS, IDs d'unité avion/carrier et jeton pilote anonymisé (**jamais l'UCID**), et les actions de
scénario attendues écrites **avant** de regarder les résultats. Une correspondance jeton↔participant
ne doit être conservée que si nécessaire, jamais partagée ; aucun UCID dans fixtures, documentation,
PNG, ACMI ou tickets.

Une preuve live n'est suffisante que si des runs répétés s'accordent et que les versions/hash
capturés sont identiques d'un run à l'autre. Ne promouvoir en fixture déterministe anonymisée
qu'à cette condition, en consignant explicitement ce que la preuve démontre et ce qu'elle ne
démontre pas. Un test de robustesse logiciel (absence/duplication/délai/réordonnancement simulés)
reste une simulation conservative, jamais une preuve de comportement DCS réel.

### Invariants d'optimisation (à respecter avant toute promotion de performance)

Une optimisation n'est acceptable que si elle ne dégrade, sur aucun scénario testé, aucun des
éléments suivants : échantillons manquants/invalides/périmés, skew/gap maximum ou p95, erreurs de
corrélation ou sorties dupliquées, interaction entre recoveries, débordement de file, redémarrages
de tâche de recovery. La cadence active 10-20 Hz de la zone de notation est fixe ; un remplacement
de la découverte/pré-filtrage par `StreamUnits` (côté fork) n'est acceptable qu'après une expérience
mesurée, jamais par défaut. Un éventuel cache de transform partagé devrait être indexé par unité,
imposer sa propre fraîcheur, et ne jamais réutiliser une donnée à travers une coupure de session/
génération ou de RPC.

## Respawns et isolation

Le registre stocke session, génération, IDs et noms. Un avion ou carrier de même nom avec nouvel ID
annule toutes les tâches de l'ancienne incarnation dans la même session/génération. Les autres
unités et générations restent isolées. Les guards prioritaires sont relâchés à l'abort. La
suspension des détecteurs reste limitée au même avion (JSON : `detector_suspension_scope:
same_aircraft`), donc une autre recovery simultanée peut être découverte.

Clé de corrélation d'une recovery (supervision/registre, distincte du `recovery_id` de nommage de
fichier plus bas) : `session + génération + ID unité avion + identité interne pilote + ID unité
carrier + mode de recovery`. Identité interne : humain résolu = UCID du slot réseau occupé (jamais
le nom d'affichage) ; humain non résolu = clé locale session/unité ; IA = `ai:<session>:<unit_id>`
(`src/commands/run.rs`). Ces clés privées ne sont jamais journalisées ni placées dans un artefact
public.

Matrice de compatibilité stricte (paire sans entrée = aucun détecteur créé) : AV-8B NA + LHA Tarawa
= V/STOL ; avion à crosse supporté + géométrie Nimitz/Forrestal = arrêté ; AV-8B NA + porte-avions
arrêté, ou avion à crosse + Tarawa = incompatible. Découverte initiale et événements `Birth`
ultérieurs utilisent la même matrice, quel que soit l'ordre d'apparition avion/navire.

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

- Nom de fichier de base réellement écrit sur disque (`src/tasks/record_recovery.rs`) :
  `LSO-<horodatage wall-clock>-<nom pilote assaini ou "unknown">-<...>`, distinct du `recovery_id`
  logique ci-dessus — évite toute collision entre deux passes simultanées y compris à l'affichage.
- Aucun `try_exists + rename`.
- Écriture dans un temporaire du même dossier, flush + `sync_all`, puis `hard_link(temp,
  destination)`.
- Création atomique sans remplacement sur Windows et Unix.
- Temporaires et répertoires de rendu nettoyés.
- Claim process-scoped `(out_dir, recovery_id)` contre deux noms concurrents.
- Le JSON identifie le producteur gagnant ; seul lui poursuit ACMI, SQLite, rendu, session log et
  Discord.
- Un artefact existant n'est jamais remplacé.

Capacités des buffers bornés en mémoire (`src/track.rs`) : `datums`/`pattern_datums` 72 000
échantillons chacun (`MAX_TRACK_SAMPLES`), `trajectory_deviations` 4 000 (`MAX_TRAJECTORY_SAMPLES`),
preuves d'événements 256 (`MAX_EVENT_EVIDENCE`), observations de hook 512 (`MAX_HOOK_EVIDENCE`) ;
file du superviseur 16 (`queue_high_watermark`, voir "Observabilité runtime"). Un dépassement est
explicite, compté, et bascule la complétude en `BufferLimit` — jamais silencieux.

SQLite : migrations additives 2–6 (`schema_migrations`), index unique partiel `recovery_id`,
`INSERT OR IGNORE`. Discord seulement pour une nouvelle ligne. UCID uniquement SQLite/API privée,
jamais JSON/PNG/ACMI/Discord/log public. Dashboard loopback `127.0.0.1`, sans OAuth/TLS, privé
phase 1.

Contenu des migrations (`src/db.rs`) : 1 = table `passes` historique ; 2 = champs
recovery/session/carrier/complétude/provenance du brin + index unique de recovery ; 3 =
`points_awarded` (sépare un vrai zéro d'une absence de points) ; 4 = spot visé, spot actif le plus
proche et distance au spot visé, séparés ; 5 = gap du segment noté, santé télémétrie, confiance de
l'estimation de brin et disponibilité de la note ; 6 = causes secondaires encodées JSON (`cause`
legacy reste la colonne primaire). Au démarrage, chaque `ALTER TABLE` est précédé d'une inspection
`PRAGMA table_info(passes)` ; une erreur de migration inattendue est retournée, jamais avalée comme
une simple "colonne déjà existante". Les lignes existantes sont préservées. `points_awarded` vaut
`true` par défaut pour les lignes historiques (une note numérique existait toujours avant ce champ)
; une nouvelle ligne incomplète stocke `0` dans la colonne legacy non-nullable `grade_points` (pour
compatibilité SQL) mais `points_awarded = false` — l'API sérialise ce dernier champ, jamais un zéro
fabriqué à partir de `grade_points` seul.

Avant tout futur changement de schéma ou nettoyage destructeur : conserver des fixtures de la base
et du JSON les plus anciens réellement rencontrés en production, et tester la migration forward et
l'affichage dashboard dessus avant de merger.

## Provenance Git et baseline

`build.rs` injecte commit et dirty. Dirty = `git status --porcelain=v1 --untracked-files=no` :
modifications/suppressions/staging suivis participent ; fichiers non suivis et `target/` ne
participent pas. Tous les chemins suivis, index, HEAD et ref active déclenchent Cargo ; `build.rs`
et `build_support.rs` ont des triggers explicites. `tests/build_provenance.rs` teste la logique
déterministe.

`--baseline-manifest` refuse objet vide, clés inconnues, valeurs vides et SHA-256 mal formés. Les
erreurs affichent chemin, ligne, colonne et cause système. Manifeste typé optionnel fourni par
l'opérateur : build DCS, versions mission/module, et SHA-256 de la mission et du DLL/Lua DCS-gRPC
déployé. Une valeur inconnue reste absente plutôt que déduite.

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
vérification a posteriori de l'éligibilité `_OK_`.

SQLite utilise le vocabulaire snake_case du JSON. L'absence d'un nouveau champ signifie
legacy/unknown, jamais favorable. `points_awarded` (`src/db.rs`, booléen) distingue explicitement
"aucun point attribué" (outcome non éligible) d'un vrai zéro de points — ne pas confondre les deux
côté API/dashboard.

## Build

Prérequis : toolchain Rust stable (édition 2021, pas de `rust-toolchain.toml` figé) et le checkout
frère `E:\DCS stuffs\Initiative ESG\DCS-gRPC` présent au même niveau que ce dépôt — `Cargo.toml`
résout `dcs-grpc-stubs` via un chemin local (`[dependencies.stubs] path = "../DCS-gRPC/stubs"`,
voir "DCS-gRPC et dépendances" plus haut) : sans ce dossier frère, le build échoue à la résolution
des dépendances, pas seulement à l'exécution.

- Build direct : `cargo build --release` (ou sans `--release` pour un binaire debug non optimisé)
  depuis la racine du dépôt ; produit `target/release/lso.exe` (ou `target/debug/lso.exe`).
- Script Windows fourni, [build.ps1](build.ps1) (non suivi par git, local) :
  `./build.ps1` (release par défaut) ou `./build.ps1 -Configuration debug` — exécute
  `cargo build --locked` (+ `--release` sauf en debug) et affiche le chemin du binaire produit en
  cas de succès ; utile pour toujours builder avec `Cargo.lock` figé (`--locked`) sans y penser.
- Vérification complète avant commit (celle que la CI exécute, voir plus bas) :
  `cargo test --locked --no-fail-fast`, `cargo fmt --check`, `cargo clippy --locked --all-targets
  -- -D warnings`, puis `git diff --check` (espaces/fins de ligne en conflit).
- Lancer le binaire construit : voir [README.md](README.md), "Quick start", pour les options CLI
  (`lso run -o <dossier>`, variable `DCS_GRPC_API_KEY`, etc.) — non dupliqué ici.
- Rejeu hors-ligne d'un ACMI déjà produit par LSO (pas un fichier Tacview quelconque) :
  `lso.exe file <chemin.zip.acmi>`, ex. `lso.exe file tests\recordings\wire_3_01_T45.zip.acmi`
  (fixtures de test réellement présentes dans `tests/recordings/`). Une invariance live/replay est
  couverte par un test, mais le replay ne peut pas reproduire le timing réseau, l'UCID, la livraison
  d'événements DCS ni la performance serveur.

## Déploiement et rollback

Runbook opérationnel (jamais exécuté en conditions réelles chronométrées à ce jour — voir
[tasking-roadmap.md](tasking-roadmap.md)). La phase 1 ne déploie ni ne modifie DCS-gRPC, le
protobuf, le DLL ou le Lua.

**Préparer un candidat** : exécuter les vérifications obligatoires (voir "Build"), builder avec
`cargo build --release --locked`, noter le SHA-256 de `lso.exe` et la révision/diff Git, copier le
candidat dans un dossier versionné sans jamais écraser le binaire actif, sauvegarder `lso.db`
(sauvegarde SQLite ou copie fichier process arrêté), conserver le binaire précédent et sa
configuration à côté du candidat. Layout suggéré :

```text
C:\LSO\releases\<revision>\lso.exe
C:\LSO\releases\previous\lso.exe
C:\LSO\data\lso.db
C:\LSO\active.txt
```

Le dossier de sortie reste partagé car les migrations sont additives — mais tester l'ancien binaire
contre une copie de la base migrée avant tout changement en production ; s'il ne sait pas lire les
colonnes additives, le pointer vers la sauvegarde pré-changement lors d'un rollback.

**Bascule** : arrêter uniquement le process/service LSO et attendre sa sortie, mettre à jour le
chemin de l'exécutable ou le pointeur de version, démarrer et vérifier sous deux minutes : connexion,
version/session serveur rapportée, aucune erreur de migration, nombre de paires strictes attendu,
dashboard sur `127.0.0.1`, log de métriques à 10 s. Rollback immédiat d'acquisition de position sans
changer de binaire : redémarrer avec `--position-source unary` ; `--legacy-inline-hook-sampling`
restaure indépendamment l'ancien chemin hook bloquant. Conserver les logs bufferisé et indépendant
avant la bascule pour garder les percentiles A/B comparables. Le candidat bufferisé exige le
DLL/Lua/protobuf DCS-gRPC exactement de la même ligne d'API que celle validée à l'exécution (voir
"DCS-gRPC et dépendances", `dcs_grpc_compatibility`) — ne jamais mélanger avec un DLL/Lua d'une autre
ligne, ni déployer seulement une moitié du paquet serveur.

**Rollback en moins de cinq minutes** : arrêter LSO, pointer vers le binaire précédent préservé, si
un test de compatibilité l'a exigé écarter la base de la tentative ratée et restaurer la sauvegarde
pré-changement (sans jamais écraser la copie ratée avant diagnostic), redémarrer l'ancien binaire,
confirmer connexion gRPC/session ID/dossier de sortie/dashboard loopback, consigner heures UTC,
hashs de binaire et raison. Un rollback réussi se mesure à la reprise de l'enregistrement local ;
un échec Discord/PNG est secondaire et ne doit jamais retarder la restauration de la persistance
locale.

Non validé à ce jour : cette procédure n'a jamais été chronométrée sur une copie de
staging avec le vrai wrapper de service et les permissions filesystème réelles — un runbook écrit
seul ne valide pas l'objectif des cinq minutes.

## CI et sécurité

`.github/workflows/ci.yml` exécute build/test avec `--locked`, Clippy `--locked --all-targets --
-D warnings`, rustfmt, installation épinglée de `cargo-audit 0.21.2 --locked`, puis `cargo audit`.
Ne jamais modifier silencieusement `.cargo/audit.toml`.

## Fichiers de contexte associés

- [primer.md](primer.md) : explication human-first, vulgarisée, du fonctionnement complet du
  module (DCS / DCS-gRPC / DCS-gRPC-lso, les 8 étapes d'un appontage noté), et de la logique de
  notation en détail vulgarisé.
- [tasking-roadmap.md](tasking-roadmap.md) : roadmap technique, décisions ouvertes et bugs connus
  non résolus ou non revalidés — voir "Règles de maintenance des documents markdown racine"
  ci-dessus.
- [CHANGES.md](CHANGES.md) : historique changelog synthétique de toutes les versions.
- [README.md](README.md) : usage utilisateur (installation, options CLI).
- [VSTOL.md](VSTOL.md) : spécification V/STOL AV-8B/Tarawa.

Le dossier `docs/` ne contient plus de documentation markdown maintenue : son contenu utile a été
consolidé ici au fil du ménage documentaire (voir [CHANGES.md](CHANGES.md) pour la trace de ce qui a
été fusionné et retiré).
