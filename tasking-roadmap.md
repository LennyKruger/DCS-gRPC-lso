# Tasking & roadmap — DCS-gRPC-lso

> Historique synthétique des chantiers, décisions ouvertes et bugs connus. Complète
> [AGENTS.md](AGENTS.md) (état courant du code) et [primer.md](primer.md) (vulgarisation). Fusionné
> le 4 septembre 2026 depuis `.ignore/tasking-v3.md`, `.ignore/roadmap-post-test-20260904.md` et les
> autres notes de session (avant leur suppression du dépôt), puis mis à jour au fil des sessions
> suivantes — dernière mise à jour le 5 septembre 2026 (soir).

## À faire en priorité (P0)

Aucun point P0 ouvert au 5 septembre 2026 (soir) : les deux bugs confirmés le matin même (désync
`wire_estimated`/`wire_estimation`, explosion `atan2` de `trajectory_deviations`) et celui confirmé
lors du test live du soir même (amplification géométrique de la trajectoire continue près du
toucher) sont corrigés — voir "Décisions déjà prises" ci-dessous. Comme pour le correctif
remise-de-gaz-en-survol, aucun des trois n'est encore revalidé sur un enregistrement live postérieur
à son propre correctif — voir "Décisions encore ouvertes".

## À investiguer — pas encore un bug confirmé (P1)

- **Écart latéral quasi constant (~0,75-0,85 m) présent à toute distance, sur les 5 appontages du
  test live du soir du 5 septembre 2026, réussis comme Cut.** Repéré en creusant le correctif
  `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` (voir "Décisions déjà prises" ci-dessous) : en reconstituant
  l'écart réel en mètres depuis `lineup_deg`/`distance_m` du JSON, l'écart latéral ne grossit pas
  proportionnellement à la distance (ce qu'on attendrait d'une vraie erreur d'alignement ou d'un
  axe de référence mal orienté) — il reste quasiment constant de 50 m jusqu'au toucher, sur les
  5 appontages sans exception. Ressemble davantage à un décalage fixe de point de référence (crosse
  vs CG projetée, ou point de visée supposé) qu'à un vrai écart d'alignement croissant. N'affecte
  pas la note de façon significative (le nouveau plancher de 75 m le neutralise déjà en grande
  partie) et n'a donc aucune urgence, mais mérite d'être creusé séparément si l'occasion se
  présente — cause non identifiée à ce stade.
- **Possible biais systématique de -1 brin dans l'estimation Rust du câble** (observé le 5
  septembre 2026, test CVN-72 du soir). Sur les 5 appontages complets, 2 divergences
  `wire_estimated`/`wire_dcs` (F14-4-1 : Rust 1 vs DCS 2 ; F18-2-1 : Rust 2 vs DCS 3) — dans les
  deux cas, l'estimation Rust (`continuous_hook_plane_crossing`) est exactement un brin en dessous
  de la valeur DCS/LQM. Seulement 2 échantillons, insuffisant pour conclure à un biais systématique
  plutôt qu'une coïncidence, mais le garde-fou existant (`wire_primary: dcs_lqm`, jamais
  l'estimation Rust affichée au pilote en cas de divergence) a fonctionné correctement dans les deux
  cas — aucun risque utilisateur immédiat, seulement une piste de précision à vérifier sur plus de
  données (un décalage constant suggérerait un franchissement de brin détecté en retard plutôt
  qu'un bruit aléatoire).
- **Déclin de l'AoA corrigée dans le groove** (~1,5–3° entre ¾ NM et ¼ NM, systématique sur 3-4
  F-14 dans un test avec vent nul). Le vent nul dans cette mission exclut un artefact de la
  correction vent introduite par le commit `108ffa1` ; reste à savoir si c'est un comportement de
  pilotage IA réel ou un effet de la décomposition en repère avion. Nécessite un outil de
  comparaison ancienne/nouvelle formule sur les mêmes données.
- **Délai fixe touchdown → fichiers complets** (~10,2 s, ±0,03 s observé) — trop régulier pour être
  du temps de calcul variable ; sent le délai intentionnel dans le pipeline (confirmation finale du
  brin ? cadence de lecture du buffer côté fork ?). Source exacte non identifiée.
- **Mémoire du process `DCS_server` en légère hausse continue** (+8,3 Mo de working set en ~6 min
  sur la session du 5 septembre 2026, 3953,4 → 3961,7 Mo, jamais redescendue). Suivi démarré en
  cours de session (pas de baseline dès le lancement du serveur, ni de corrélation fine
  appontage-par-appontage) — voir historique des sessions. Insuffisant pour conclure à une vraie
  fuite mémoire côté DCS (pas de preuve qu'un appontage la libère jamais, mais pas non plus assez de
  recul pour l'affirmer). LSO n'écrit rien côté DCS et ne peut qu'observer depuis l'extérieur (compteur
  Windows du process) ; à refaire sur une session dédiée, plus longue, avec mesure dès le démarrage
  du serveur.

## Optimisations à considérer (P2)

- **Boucle de redémarrage pendant une pause mission** (observé : 6 redémarrages de génération en
  ~30 s, un par timeout de session ID). Sans gravité mais bruyant ; un backoff progressif
  réduirait le bruit de logs lors d'une pause prolongée.
- **Vent nul non exercé en test** : la mécanique de capture (2 requêtes `AtmosphereService.GetWind`
  à l'entrée du groove) a été validée mécaniquement, mais le calcul de correction lui-même (la
  soustraction du vecteur vent) n'a jamais été vraiment exercé avec du vent réel. À refaire avec du
  vent configuré dans la mission de test.
- **Purger le ring Lua sur `after_sequence` acquitté** : déjà implémenté (voir AGENTS.md,
  section DCS-gRPC), à revalider en usage prolongé.
- **`telemetryObservationErrors` bornée à 128 entrées** : déjà implémenté, à revalider.
- **Tentatives d'approche avorté avant le groove, totalement invisibles hors logs DEBUG.** Observé
  le 5 septembre 2026 : `F18-3-1` a déclenché deux détections de pattern réelles
  (`found pattern / recovery attempt`) qui se sont soldées par `discard as plane was never below
  100m MSL` — aucun JSON, aucune ligne SQLite, rien dans le tableau de bord. Sur le principe c'est
  correct (rien à noter tant que l'avion n'est jamais descendu assez bas), mais ça veut dire qu'un
  avion qui enchaîne les tentatives avortées (deck foul suspecté côté mission ce jour-là, voir la
  note opérationnelle ci-dessous) n'apparaît nulle part dans les artefacts destinés au pilote/LSO
  humain — seul quelqu'un qui lit `-v`/`-vv` en direct le voit. Piste : un compteur minimal
  (nombre de tentatives détectées puis avortées avant groove, par avion/session) exposé quelque
  part de plus visible que le log DEBUG, sans pour autant fabriquer un rapport pour une approche qui
  n'a jamais existé au sens du grading.

## Hors-scope confirmé (rappel volontaire)

- **Robustesse multi-recoveries simultanées** : la dernière mission de test était trop
  minimaliste (8 unités, jamais deux avions en approche en même temps) pour être testée.
- **Cadence adaptative pré-groove (100/200 ms)** : toujours en attente d'une décision explicite,
  indépendante de tout test. `lso.exe cadence-ab` (voir AGENTS.md) est l'outil de mesure prévu pour
  instruire cette décision, pas la décision elle-même. Exécuté une fois sur un corpus de 9 rapports
  live : aucun changement de grade, mais 2 occurrences où la porte ¾ NM devient invalide (strides 2
  et 4) sur l'enregistrement le plus récent, déjà sous-échantillonné côté capture — confirme
  empiriquement le risque déjà identifié (un sous-échantillonnage supplémentaire cumulé casse la
  porte la plus proche du groove).
- **Nouveau message `RecoveryTelemetry` compact** : changement de protocole additif, deux dépôts,
  régénération des stubs — chantier séparé, non entamé.

## Note opérationnelle (procédure de test, pas un bug LSO)

Arrêter une tâche de surveillance qui encapsule `lso.exe` dans son propre pipeline tue aussi
`lso.exe` — a coûté l'enregistrement d'un trap en cours d'approche lors d'un test. À éviter dans une
future session : lancer le process et le monitoring séparément.

**Foul deck suspecté côté mission, pas côté LSO (5 septembre 2026).** En fin de session,
`F18-4-1` a enchaîné 3 waveoffs consécutifs coupés systématiquement à ~165 m du pont (`WO?`, DCS
`GRADE:WO WOFDIC` à chaque fois), et `F18-3-1` n'est jamais descendu sous 100 m MSL sur ses deux
tentatives (voir P2 ci-dessus). Schéma cohérent avec un pont indisponible (gear pas en batterie,
zone d'appontage occupée) plutôt qu'avec un problème de pilotage IA répété. DCS-gRPC n'expose
aujourd'hui aucun signal d'état du pont/de la passerelle arrière ; LSO ne peut donc ni confirmer ni
infirmer cette hypothèse et documente fidèlement "remise de gaz, initiateur inconnu" à chaque fois,
comme prévu par conception (voir AGENTS.md, "jamais inventer OWO/WOP/waveoff pilote"). À vérifier
directement dans la mission/le jeu lors d'une prochaine session, pas par une modification de LSO.

## Décisions déjà prises (ne pas rouvrir sans nouvelle preuve)

Ces points étaient listés "ouverts" dans les versions antérieures de la roadmap (`tasking-v3.md`) ;
ils sont désormais tranchés et implémentés :

- **Batch vs stream source** : batch incrémental `after_sequence`, implémenté et actif par défaut
  (`--position-source buffered`).
- **Hook groupé ou indépendant** : resté totalement indépendant du snapshot avion/carrier.
- **Partage carrier en multi-recovery** : clé `(session, generation, carrier_id, sequence)`, jamais
  de cache périmé dans une preuve de gate.
- **Format de causes multiples** : `cause` reste l'alias primaire, `causes: { primary, secondary[] }`
  ajouté en JSON, migration SQLite 6 additive.
- **AoA réelle via draw argument (`aircraft_draw_argument`/`DrawArgumentObservation`)** : abandonné
  après recherche sérieuse infructueuse sur les vraies valeurs de draw argument par module (sources
  web bloquées/ambiguës). Remplacé par une correction du vent sur l'approximation géométrique
  existante (commit `108ffa1`), documentée `PROJECT-DERIVED`. Ne pas relancer cette piste sans accès
  DCS live pour un balayage empirique de `UnitService.GetDrawArgumentValue`.
- **Remise de gaz en survol classée `Bolter` au lieu de `WO?`** : `crossed_deck_threshold` dans
  [src/track.rs](src/track.rs) ne marque désormais le franchissement du seuil de pont (`x` passant
  de positif à négatif) que si l'avion est réellement proche du niveau du pont au moment du
  franchissement (`DECK_CROSSING_ALT_CAP_FT = 50 ft`, relatif au pont, crosse comprise), et non
  simplement dans les 500/300 ft déjà utilisés ailleurs (`in_approach`, entrée en groove) — ces
  seuils-là restent trop hauts pour exclure le cas confirmé en live (franchissement à ~460 ft). Deux
  tests de régression couvrent le franchissement haute altitude (→ `WaveoffUnknown`) et le
  franchissement bas niveau (→ toujours `Bolter`, garde-fou contre une sur-correction). Non encore
  revalidé en mission live — voir "Décisions encore ouvertes" et "Scénarios de validation Phase 6".
- **`wire_estimated` désynchronisé de `wire_estimation`** : `cable_estimated` n'est plus calculé
  séparément au moment du toucher (`Track::landed`) ou du déclassement en touch-and-go
  (`Track::next_sample`, cas déjà atterri puis repartant) — ces deux points ne posent plus qu'un
  `cable_estimated: None` provisoire. `Track::finish()` réconcilie ensuite systématiquement
  `cable_estimated` (pour `Grading::Recovered` et `Grading::TouchAndGo`, recoveries `Arrested`
  seulement) avec `wire_estimation.wire`, déjà calculé une seule fois à cet endroit contre
  l'historique complet de `self.wire_crossings` — élimine par construction la course entre la
  corrélation événementielle du `Land` et le tick positionnel qui ajoute le franchissement de brin
  correspondant. Un test de régression (`wire_estimated_matches_wire_estimation_even_when_the_event_path_races_ahead_of_the_crossing`,
  `src/track.rs`) rejoue exactement cette course. Non encore revalidé sur les enregistrements live
  du 5 septembre où le bug avait été observé — voir "Décisions encore ouvertes".
- **Explosion numérique de `trajectory_deviations` près du toucher** : nouvelle constante
  `TRAJECTORY_MIN_DISTANCE_M = 3.0` (`src/track.rs`) sous laquelle un échantillon n'est plus poussé
  dans `trajectory_deviations`, dans `Track::next` et dans `replay_gate_and_trajectory` (chemin
  `cadence-ab`) identiquement. `gs_deviation_deg`/`lineup_deg` restent des `atan2(écart_m, x)` bruts
  au-dessus de ce plancher — pas de changement de méthode, seulement une borne basse sur `x`. Un
  test de régression (`trajectory_deviations_stop_before_the_atan2_blow_up_near_touchdown`,
  `src/track.rs`) rejoue une approche propre jusqu'à x=0,30 m avec un flare réaliste (~0,3 m) et
  vérifie qu'aucun échantillon poussé n'est sous le plancher ni ne dépasse 45°. 3 m est
  `PROJECT-DERIVED`, choisi pour dégager confortablement la zone de blow-up observée en live (70,3°
  à x=0,30 m, 32,5° à x=1,47 m) sans retirer de couverture réelle sur l'approche — non encore
  revalidé sur les deux enregistrements live du 5 septembre où le bug avait été observé.
- **`_OK_` automatique ("passe parfaite", 5 septembre 2026)** : `is_amplitude_perfect`/
  `grade_from_gates` (`src/grading.rs`) accorde `_OK_` (5,0 points, symbole `OFFICIAL` NAVAIR
  00-80T-104 §11.4.1) uniquement depuis une passe déjà `Ok` par toutes les règles existantes
  (tendance/oscillation/fenêtre des 150 m déjà passées), si en plus (1) chaque porte et chaque
  échantillon continu reste dans une bande resserrée (`OK_PERFECT_GS_HIGH_DEG`/
  `OK_PERFECT_GS_LOW_DEG` = +0,4°/-0,3°, `OK_PERFECT_LU_ABS_DEG` = 0,5° — sans pardon sur les
  portes, avec le pardon anti-bruit habituel `PERSISTENCE_MIN_CONSECUTIVE_SAMPLES` sur la
  trajectoire continue) et (2) `groove_time_secs` tombe dans `15,0..=18,0` s
  (`OK_PERFECT_GROOVE_TIME_MIN_S`/`_MAX_S`). Recherche documentaire dédiée avant implémentation :
  aucun des deux NATOPS ne donne de seuil numérique pour "passe parfaite", mais NAVAIR 00-80T-105
  §6.2.4.3 documente bien littéralement "a 15 - 18 second groove before aircraft touchdown" — ce
  point précis, initialement écarté à tort par erreur de ma part dans cette session, a été corrigé
  après relecture directe du texte extrait du PDF. La bande d'amplitude, elle, reste
  `PROJECT-DERIVED`, reprise telle quelle du mod open-source MOOSE `Ops.Airboss` (`Airboss.lua`,
  champs `AIRBOSS.GLE`/`AIRBOSS.LUE` `_max`/`_min`) — la même filiation historique que
  `GS_SLIGHT_*`/`LU_SLIGHT`. Décision explicite de ne **pas** relier `_OK_` au brin accroché : la
  combinaison MOOSE originale "brin 3 + 15-18,99 s" (déjà désactivée dans ce projet, voir plus haut)
  reste désactivée, aucun texte NATOPS ne liant un brin précis à une note. Un touch-and-go est
  plafonné un cran sous `grade_from_gates`, ne peut jamais recevoir `_OK_` (déjà documenté avant
  cette session : "A touch-and-go cannot receive `_OK_` or points"). Limite assumée : la fenêtre
  15-18 s est appliquée telle quelle à tous les CATOBAR (F-14/F-18/T-45) malgré la pente différente
  du T-45 (3,0° vs 3,5°), faute de référence de vitesse d'approche par type dans le modèle de
  données du projet — option retenue explicitement par l'utilisateur plutôt que d'exclure le T-45
  ou de ne garder le critère qu'informatif. `lso.exe cadence-ab` ne rejoue pas la détection de
  toucher, donc `groove_time_secs` y vaut toujours `None` : `_OK_` n'apparaît jamais dans une note
  rejouée par ce diagnostic, seulement en usage live. Sept tests de régression ajoutés
  (`src/grading.rs`) : passe propre + temps correct → `_OK_` avec brin 3 puis brin 4 (preuve
  d'indépendance au brin), bornes 15,0/18,0 s inclusives, bornes 14,99/18,01 s exclusives,
  touch-and-go jamais `_OK_` même à amplitude et temps parfaits, une seule porte hors bande refuse
  `_OK_` sans affecter `OK`, un pic isolé de trajectoire est pardonné, deux pics consécutifs
  refusent `_OK_` sans affecter `OK`. Deux tests existants
  (anciennement `legacy_wire3_time_window_does_not_upgrade_to_perfect` et
  `clean_wire4_pass_remains_ok`) verrouillaient l'ancien comportement "jamais de `_OK_` automatique"
  ; ils ont été réécrits pour vérifier le nouveau comportement intentionnel (`_OK_` atteint,
  indépendamment du brin) plutôt que supprimés. `cargo test --locked --no-fail-fast` (193 réussis,
  191 tests du binaire + 2 tests de provenance de build), `cargo fmt --check` et `cargo clippy
  --locked --all-targets -- -D warnings` propres. Aucune preuve DCS live — voir "Décisions encore
  ouvertes".
- **Calibration de la crosse (T&G vs Bolter) étendue au T-45 et au F-14 (5 septembre 2026)** :
  jusqu'ici, `Track::calibrated_hook_state` (`src/track.rs`) ne savait interpréter la position de
  crosse (lue via `UnitService.GetDrawArgumentValue`) que pour le F/A-18C ; tout autre type
  retombait systématiquement sur `Bolter`, même pour un vrai touch-and-go volontaire, faute de
  connaître l'index de draw argument à interroger. L'utilisateur a fourni les deux index manquants :
  **25** pour le VNAO T-45 (identique au F/A-18C) et **1305** pour le F-14 (les trois variantes
  A/B/B(U), un index commun distinct). Nouveau champ `AirplaneInfo::hook_draw_argument: Option<u32>`
  (`src/data.rs`) : `Some(25)` pour F/A-18C et T-45, `Some(1305)` pour les F-14, `None` pour l'AV-8B
  (pas de crosse dans ce workflow). `calibrated_hook_state` n'exige plus un nom de type précis
  ("F/A-18C Hornet") mais la seule présence d'un index connu — n'importe quel type sans index reste
  `Unknown`, jamais un `TouchAndGo` inventé. Les deux points d'interrogation gRPC
  (`src/tasks/record_recovery.rs`, chemin indépendant `sample_hook` et chemin `LegacyInline`)
  interrogent désormais l'index propre au type plutôt que `25` en dur.

  **Limite assumée et documentée** : seuls les seuils d'interprétation eux-mêmes (`<= 0,2` = up,
  `>= 0,8` = down) ont été confirmés empiriquement pour le F/A-18C
  (`HookObservation::polarity = "fa18c_zero_up_one_down_test_corpus"`). Pour le T-45 et le F-14,
  l'utilisateur a fourni l'index de draw argument mais pas de confirmation indépendante que la même
  convention 0/1 s'applique — le code réutilise la même polarité par hypothèse
  (`"assumed_zero_up_one_down_pending_live_validation"`), pas par preuve. Deux tests existants
  réécrits : l'ancien `uncalibrated_f14_hook_values_remain_unknown` (qui verrouillait l'ancien
  comportement "F-14 toujours Unknown") est devenu `f14_hook_is_calibrated_via_its_own_draw_argument`
  (vérifie que le F-14 atteint désormais `Up`/`Down` comme le F/A-18C) ; un nouveau test
  `type_without_a_known_hook_draw_argument_remains_unknown` couvre le cas générique "type sans
  index connu" avec un `AirplaneInfo` fabriqué pour le test, afin de ne pas perdre la couverture de
  ce garde-fou. `cargo test --locked --no-fail-fast` (194 réussis, 192 tests du binaire + 2 tests de
  provenance de build), `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings`
  propres. Aucune preuve DCS live — la polarité T-45/F-14 reste à confirmer en priorité (voir
  "Décisions encore ouvertes"), un vrai touch-and-go sur l'un de ces deux types pourrait encore être
  mal classé si la convention supposée s'avère fausse pour son modèle.
- **`groove_time_secs` exposé dans le JSON (5 septembre 2026, suite au test live CVN-72 du soir)** :
  ce champ, l'une des deux conditions de `_OK_` automatique (voir "Automatic `_OK_`" dans
  [docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md)), était déjà calculé
  (`Track::groove_time_secs`) mais uniquement affiché dans l'embed Discord
  (`src/tasks/record_recovery.rs`, commentaire "Discord-only fields") — jamais sérialisé dans le
  rapport JSON. Repéré lors du test live du soir : un appontage noté `_OK_` par DCS lui-même
  (F14-2-1, brin 3) n'a laissé aucune trace de sa durée de groove dans son JSON, faute de Discord
  configuré pour ce test, rendant impossible toute vérification a posteriori de l'éligibilité
  `_OK_`. Corrigé en ajoutant `groove_time_secs: Option<f64>` à `RecoveryReport`
  (`src/tasks/record_recovery.rs`), peuplé directement depuis `track.groove_time_secs` ; aucun
  changement au calcul lui-même ni à l'embed Discord existant. `cargo test --locked --no-fail-fast`
  (194 réussis, 192 tests du binaire + 2 tests de provenance de build), `cargo fmt --check` et
  `cargo clippy --locked --all-targets -- -D warnings` propres. `cadence-ab` n'est pas concerné (ne
  construit pas `RecoveryReport`, `groove_time_secs` y reste toujours absent par construction — la
  détection de toucher n'y est jamais rejouée).
- **`NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` (5 septembre 2026, suite au P0 confirmé lors du test live
  CVN-72 du soir même)** : `gs_deviation_deg`/`lineup_deg` de `trajectory_deviations` sont
  `atan2(écart_m, x)` ; `TRAJECTORY_MIN_DISTANCE_M = 3 m` empêchait déjà l'explosion pure vers
  l'infini quand `x -> 0`, mais une version plus modérée (et tout aussi trompeuse) du même effet
  restait active bien au-dessus de ce plancher : un écart quasi constant de quelques décimètres à
  1 m (confirmé présent dès 50 m, verticalement et latéralement, sur les 5 appontages du test —
  propres comme Cut) grossissait mécaniquement à deux chiffres en degrés dans les derniers mètres,
  simplement parce que `x` rétrécit, faisant retomber deux passes par ailleurs irréprochables
  (dont une notée `_OK_` par DCS) à `NoGrade`, et faisant franchir le seuil de Cut GS (-2,5°) à un
  appontage dont le biais bas réel (~1,4-1,7 m) était en fait stable, voire en légère amélioration,
  depuis ~110 m — reconstitution détaillée en repartant du JSON (`distance_m * tan(gs_deviation_deg)`
  pour retrouver l'écart réel en mètres à chaque échantillon).

  Corrigé en substituant une distance de référence fixe à `x` dans l'appel `atan2` dès que `x`
  passe sous ce seuil (`x.max(NEAR_TOUCHDOWN_ANGLE_REFERENCE_M)`), via une nouvelle fonction
  partagée `trajectory_deviation_angles_deg` (`src/track.rs`) appelée identiquement par
  `Track::next` et `replay_gate_and_trajectory` — même précaution que pour l'ancien bug
  `wire_estimated`/`wire_estimation` : un seul point de calcul pour les deux chemins, jamais deux
  formules qui pourraient diverger. Au-dessus de la distance de référence, `x.max(...)` ne change
  rien (elle vaut déjà `x`) : le comportement pour le reste de l'approche est identique à avant. Un
  écart réel et important garde toute sa capacité à dégrader la note ou déclencher un Cut — il lui
  faut désormais un nombre de mètres réel pour cela, plus seulement quelques décimètres combinés à
  la proximité du pont (`atan2(-3,5 m, 75 m) ≈ -2,67°`, toujours un Cut).

  `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75,0` (mètres) est `PROJECT-DERIVED`, choisi comme
  approximativement une seconde de vol à la vitesse d'approche CATOBAR typique (~75 m/s) :
  confortablement plus grand que les écarts de flare/référence confirmés en live divisés par les
  seuils angulaires `Ok`/Cut, tout en restant assez court pour qu'un écart ne se développant
  vraiment que dans les toutes dernières secondes garde sa propre géométrie réelle plutôt que
  d'être évalué comme s'il était encore à une distance de porte. Deux nouveaux tests de régression
  (`src/track.rs`) : `near_touchdown_flare_offset_no_longer_manufactures_a_large_angle` (un écart
  constant de 0,8 m de 50 m à 4 m reste sous 1°/2° au lieu d'exploser) et
  `near_touchdown_large_offset_still_crosses_the_cut_threshold` (un écart réel de 3,5 m déclenche
  toujours le Cut). `cargo test --locked --no-fail-fast` (196 réussis, 194 tests du binaire + 2
  tests de provenance de build), `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D
  warnings` propres. Détail complet dans
  [docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md), "Near-touchdown geometry" ; résumé
  vulgarisé dans [docs/NOTATION-LOGIQUE.md](docs/NOTATION-LOGIQUE.md). Aucune preuve DCS live sur
  ce correctif précis — voir "Décisions encore ouvertes".

## Décisions encore ouvertes (mission/serveur nécessaires)

- Revalider en mission live `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75 m` : corrigé et testé
  unitairement (deux nouveaux tests synthétiques) et vérifié par reconstruction manuelle contre les
  5 rapports JSON du test du 5 septembre 2026 (soir), mais jamais encore rejoué en conditions live
  pour confirmer qu'il replace bien F14-1-1/F14-2-1 en `Ok`/`(OK)` et retire bien le Cut artefactuel
  de F14-4-1/F18-2-1 sans introduire de nouveau faux négatif (un écart réellement dangereux et bref,
  développé uniquement dans les tout derniers mètres, pourrait en théorie être un peu moins vite
  détecté qu'avant — à vérifier sur un cas réel, aucun des 5 appontages observés n'en fournissait un
  exemple net). 75 m n'a par ailleurs aucune base doctrinale chiffrée (comme le Cut sink-rate/bank) :
  choisi par cohérence physique (~1 s de vol à l'approche), pas calibré sur un corpus large.
- Le collecteur `--positions-only` atteint-il réellement p99 <300 ms en usage prolongé et
  multi-recovery ?
- Cadence de lecture LSO adaptative 100/200 ms hors zone de notation : à valider par A/B
  (`lso.exe cadence-ab`) avant toute promotion — voir ci-dessus, jamais décidée à ce jour.
- Validation live DCS-gRPC de la ligne de version serveur réellement déployée avant tout repin des
  stubs vers une révision Git immuable.
- Refaire les scénarios de validation Phase 6 (voir plus bas) sur une mission moins minimaliste :
  câbles 1–4, pattern puis finale, longue passe, simultané, respawn, reconnect/session, gaps, sans
  ACMI et V/STOL.
- Revalider en mission live le correctif de la remise de gaz en survol (`DECK_CROSSING_ALT_CAP_FT`) :
  corrigé et testé unitairement, mais jamais encore rejoué sur un vrai enregistrement CVN-72 pour
  confirmer que les 8 tentatives de F18-4-1 du test du 4 septembre 2026 seraient bien reclassées.
- Revalider sur données live les nouveaux seuils de notation (`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`,
  `OSCILLATION_MIN_SWING_DEG`/`OSCILLATION_MIN_REVERSALS`) : corrigés et testés unitairement
  seulement, jamais rejoués sur un corpus de rapports live pour vérifier qu'ils ne masquent pas de
  vrais écarts ni ne déclenchent de faux positifs sur des approches réelles.
- Revalider sur données live le raffinement d'entrée en groove CATOBAR
  (`GROOVE_ROLLOUT_MAX_BANK_DEG`/`GROOVE_ROLLOUT_MAX_TRACK_ANGLE_DEG` = 15° chacun) : testé
  unitairement seulement (approches synthétiques). Reste à confirmer que 15°/15° ne retarde jamais
  une vraie entrée en groove sur un virage large ou un fort vent de travers, et qu'ils excluent bien
  un survol transitoire de la boîte pendant le virage 180→90 sur un enregistrement réel.
- `cargo audit` reste à exécuter dès qu'un outil autorisé est disponible localement (la CI
  l'exécute déjà).
- Revalider sur les enregistrements live du 5 septembre 2026 le correctif de la désynchronisation
  `wire_estimated`/`wire_estimation` (`s1788591037-g8-p8-c1-t929340`, `s1788591037-g8-p9-c1-t1180160`)
  et celui de l'explosion `atan2` de `trajectory_deviations`
  (`s1788591037-g8-p2-c1-t211460`, `s1788591037-g8-p6-c1-t458200`) : corrigés et testés unitairement
  seulement, jamais rejoués sur ces rapports live précis pour confirmer que les deux anomalies
  disparaissent bien sans effet de bord.
- Revalider sur données live le nouveau Cut sink-rate/bank-angle (`SINK_RATE_CUT_MPS = 8.0 m/s`,
  `BANK_ANGLE_CUT_DEG = 30°`) : contrairement à tous les autres seuils de cette liste, celui-ci n'a
  **aucune base doctrinale chiffrée** à valider — seulement à confirmer, sur un corpus de rapports
  live variés (dont au moins une approche par gros temps/vent de travers, où le sink rate/bank
  réels sont naturellement plus élevés), qu'il ne déclenche jamais de faux positif sur un poser
  normal ni ne manque un vrai cas dangereux. Priorité avant toute promotion : le risque de faux
  positif est le plus élevé de tous les seuils du module, faute de nombre NATOPS à recaler dessus.
  **Premier contrôle fait le 5 septembre 2026** contre les 9 rapports de
  `.ignore/live-runs/20260905-084848-buffered/` (6 posés + 3 remises de gaz) : aucun faux positif,
  et pas même un cas proche du seuil — sink rate maximum observé 6,75 m/s (84 % du seuil) et bank
  maximum 3,5° (12 % du seuil) à l'intérieur du 1/4 NM ; les trois remises de gaz montrent des
  sink rates fortement négatifs (jusqu'à -29 m/s, montée en remise de gaz) qui ne déclenchent
  jamais le Cut par construction (le contrôle ne teste que la descente). **Mais ce corpus est
  entièrement piloté par IA**, avec des approches anormalement lisses (gîte quasi nulle,
  descente très régulière) : ce contrôle prouve l'absence de faux positif sur ce corpus précis,
  pas la justesse du seuil pour un vrai cas dangereux (aucune approche de ce corpus n'approche la
  zone à risque). Reste donc entièrement à faire : un enregistrement avec un pilotage plus
  agressif ou dégradé (humain, vent fort, correction tardive) pour tester si le seuil déclenche au
  bon moment.
- Revalider en mission live l'automatisation de `_OK_` (bande d'amplitude MOOSE-inspirée +
  fenêtre de temps de groove NATOPS 15-18 s) : deux points distincts à confirmer séparément. (1)
  La bande d'amplitude resserrée n'a, comme le Cut sink-rate/bank, aucune base NATOPS chiffrée —
  à vérifier qu'elle ne déclenche jamais `_OK_` à tort sur un poser simplement propre mais pas
  réellement parfait. (2) La fenêtre de temps 15-18 s, elle, est bien `OFFICIAL`, mais appliquée
  aujourd'hui identiquement au T-45 malgré sa pente différente (3,0° vs 3,5°) — à vérifier sur des
  posers T-45 réels si cette fenêtre convient à son profil de vitesse propre ou si elle exclut à
  tort des T-45 par ailleurs parfaits (ou, à l'inverse, en admet qui ne le seraient pas dans un
  découpage NATOPS pensé pour un jet standard). Corrigé et testé unitairement seulement, jamais
  observé sur un vrai `_OK_` en conditions live — d'autant plus incertain que `_OK_` est, par
  conception, censé être rare ("Unicorn").
- Confirmer en mission live la polarité de la crosse (up/down) pour le T-45 (index de draw
  argument 25, partagé avec le F/A-18C) et pour le F-14 (index 1305, toutes variantes) : seul
  l'index a été fourni, la convention `<= 0,2` = up / `>= 0,8` = down reste une hypothèse reprise
  du F/A-18C, jamais vérifiée indépendamment pour ces deux types. Priorité : réaliser un vrai
  touch-and-go volontaire (crosse relevée) sur un T-45 puis sur un F-14 en session live, et
  vérifier dans le JSON (`hook_observation.timeline`, valeurs `raw`) que la lecture bascule bien
  vers `<= 0,2` au moment voulu — sinon la polarité doit être inversée ou recalibrée pour le(s)
  type(s) concerné(s).

## Pistes d'amélioration de la notation (état après la session du 4 septembre 2026, notation II)

Comparaison de la méthode actuelle ("trois gates + pire écart") à la doctrine LSO américaine des
années 2000 (NATOPS 00-80T-104). Conclusion : le principe fonctionne mais reste plus pauvre que ce
que les données déjà enregistrées permettraient, sans changement de DCS ni du fork. Déjà codé :
trajectoire continue, facteur de tendance, pondération temporelle des 150 derniers mètres, vent
persisté en contexte (voir AGENTS.md, grading v4), et depuis cette session : garde de persistance
(A.1), détection de surcorrection (A.4/OC), taux de descente et angle de gîte en contexte. Détail
technique dans [docs/GRADING_REFERENCE.md](docs/GRADING_REFERENCE.md).

1. **NC vs statut neutre dédié — pas de changement de code, ré-examiné et jugé déjà couvert.**
   `grading_availability` (`available` / `unavailable_technical` / `unavailable_event_outcome`)
   distingue déjà, en JSON, une `NC` d'origine télémétrique d'une `NC` d'origine événementielle ;
   introduire un nouveau statut *pass_grade* dédié à "neutre par construction" serait une décision
   de nommage/UX (quel libellé, quel impact sur le greenie board existant) plutôt qu'un correctif de
   clarté à faible risque — hors scope d'une implémentation automatique sans validation produit.
   Laissé ouvert pour une session dédiée si le besoin se confirme.
2. **AoA réellement lue depuis le cockpit** (`aircraft_draw_argument`) : voir "décisions déjà
   prises" ci-dessus — abandonné pour l'instant faute de source fiable, pas un refus définitif.
3. **Taux de descente (sink rate) — codé, contexte général + Cut dédié (5 septembre 2026).**
   `TrajectoryDeviation` porte `sink_rate_mps` (m/s, positif = descend, calculé en `d(alt)/dt`
   entre échantillons continus consécutifs) et `alt_m`. Reste hors notation sur son
   amplitude/tendance générale (pas de plafond `(OK)` façon A.2/A.4), mais alimente désormais un
   Cut dédié (`dangerous_sink_rate_or_bank`, `src/grading.rs`) : `SINK_RATE_CUT_MPS = 8.0 m/s`
   soutenu (>=3 échantillons consécutifs) à l'intérieur du 1/4 NM grade la passe `C`. Seuil
   **sans base NATOPS chiffrée** — voir point 8 ci-dessous pour la recherche documentaire qui a
   précédé cette décision.
4. **Détection de surcorrection (`OC` — overcontrolled) — codée.** Un compteur d'inversions de
   signe sur GS et lineup (`OSCILLATION_MIN_REVERSALS = 2` inversions d'au moins
   `OSCILLATION_MIN_SWING_DEG = 0.3°` chacune, sur la même fenêtre de 4 s que le facteur de
   tendance A.2) plafonne désormais une passe par ailleurs `Ok` à `(OK)`, exactement comme le
   facteur de tendance mais sensible à une pente nette proche de zéro. Seuils `PROJECT-DERIVED`,
   pas encore éprouvés sur données live.
5. **Angle de gîte (bank) aux corrections — codé, contexte général + Cut dédié (5 septembre 2026).**
   `TrajectoryDeviation` porte `bank_deg` (roll brut de la télémétrie) ; `datums[].roll_deg` porte
   la même donnée sur toute la trajectoire pour que le rejeu `cadence-ab` reste fidèle. Reste hors
   notation sur son amplitude générale, mais alimente désormais le même Cut dédié que le sink rate
   (point 3) : `BANK_ANGLE_CUT_DEG = 30°` soutenu (>=3 échantillons consécutifs) à l'intérieur du
   1/4 NM grade la passe `C`. Seuil sans base NATOPS chiffrée non plus — voir point 8.
6. **Filtre de durée minimale pour l'amplitude continue (A.1) — codé.** Un échantillon isolé
   au-dessus du seuil (`*_SLIGHT`) ne compte plus seul : il faut au moins
   `PERSISTENCE_MIN_CONSECUTIVE_SAMPLES = 2` échantillons consécutifs dans la même direction.
   Jamais appliqué au Cut (`GS_CUT_LOW_DEG`) ni à la pondération de fin d'approche (A.3), qui
   restent volontairement sensibles à un seul échantillon dangereux. Toujours non confirmé par des
   données live (aucun faux positif n'avait été observé le 4 septembre, ce correctif reste donc une
   robustesse préventive) — à revalider en mission.
7. **Entrée en groove CATOBAR affinée par roulis/route (5 septembre 2026) — codé.** Comparaison de
   la boîte géométrique historique de `entered_groove` (`x <= 3/4 NM`, `alt <= 300 ft`, lineup
   `<= ±10°`) aux deux PDF NATOPS déposés dans `.ignore/` (`CV-NATOPS-JUL09.pdf` = NAVAIR
   00-80T-105, `LSO-NATOPS-MAY09.pdf` = NAVAIR 00-80T-104) : le rayon ¾ NM de cette boîte est en
   réalité la distance de transition au contrôle LSO du **Case III** (00-80T-104 §6.6.3.1), pas un
   seuil Case I ; le Case I (00-80T-105 §6.2.4.2/6.2.4.3) définit le début du groove comme un
   événement piloté ("roll wings level on centerline with a centered ball") après le virage
   180→90→start, jamais une distance/altitude fixe, et un groove de 15-18 s dont la longueur varie
   avec la vitesse d'approche propre à chaque type. La boîte seule ne peut donc pas distinguer un
   vrai roll-out d'un survol transitoire de la boîte en pleine dernière portion du virage. Ajouté :
   `is_rolled_out()` (`src/track.rs`) exige, une fois déjà dans la boîte et sur
   `GROOVE_ROLLOUT_MIN_CONSECUTIVE_SAMPLES = 2` échantillons consécutifs, un roulis quasi nul
   (`GROOVE_ROLLOUT_MAX_BANK_DEG = 15°`) et une route sol déjà alignée sur l'axe du groove
   (`GROOVE_ROLLOUT_MAX_TRACK_ANGLE_DEG = 15°`, calculée sur la fenêtre `gate_samples` déjà
   bufferisée pour l'interpolation des gates — aucun nouveau champ de télémétrie requis, la
   fonction est donc partagée telle quelle par `Track::next` et le rejeu `replay_gate_and_trajectory`
   de `cadence-ab`). Seuils `PROJECT-DERIVED`, NATOPS ne chiffrant ni l'un ni l'autre. **CATOBAR
   uniquement** : V/STOL Tarawa garde l'ancienne boîte seule (pas de virage final CATOBAR à
   distinguer dans son profil hover/cross/VL — voir VSTOL.md), et cette géométrie de boîte reste,
   comme avant le raffinement, non pensée pour un éventuel Case II/III (non modélisé par ce projet
   — voir README.md). Voir AGENTS.md, "Gates, outcomes et câble", pour le détail à jour.
8. **Cut dédié sink-rate/bank-angle (5 septembre 2026) — codé, décision explicite malgré
   l'absence de base NATOPS chiffrée.** Suite à la question de savoir si les points 3 et 5
   participaient à la note (réponse : non, aucun des deux ne l'a jamais fait, même
   indirectement — vérifié par grep exhaustif dans `src/grading.rs`), recherche documentaire
   dédiée dans les deux PDF NATOPS de référence (extraction texte via `pdftotext -layout`) : ni
   00-80T-104 ni 00-80T-105 ne codifient de seuil numérique danger/pas-danger pour le sink rate ou
   l'angle de gîte. `TMRD` (Too Much Rate of Descent) et `W`/`TMA`/`DLW`/`DRW` (Wings/attitude/
   dropped-wing) sont des codes de commentaire qualitatifs sur la fiche de notation, au même titre
   que `LIG` ou `EG` — jamais liés à un nombre. Le seul passage substantiel (00-80T-104 §6.6.4,
   "Foul Deck Waveoff") cite sink rate et angle de bank comme des facteurs parmi d'autres du
   jugement du LSO ("aircraft/engine performance, approach dynamics, and environmental
   conditions"), exactement comme pour l'entrée en groove Case I (point 7) — même schéma :
   principe documenté, jamais de nombre. Recherche web complémentaire : régime nominal de poser
   CATOBAR sans flare ~600-800 ft/min (~3,0-4,1 m/s), aucune limite opérationnelle de sécurité
   LSO chiffrée trouvée publiquement (seules des limites structurelles de train d'atterrissage
   existent, hors sujet).

   Décision utilisateur explicite malgré ce constat : implémenter un Cut direct plutôt qu'un
   simple plafond `(OK)`. `dangerous_sink_rate_or_bank()` (`src/grading.rs`) grade `C` un sink
   rate `SINK_RATE_CUT_MPS = 8.0 m/s` (environ le double du régime nominal ci-dessus) ou une gîte
   `BANK_ANGLE_CUT_DEG = 30°` (le double de `GROOVE_ROLLOUT_MAX_BANK_DEG`) soutenus sur
   `DANGER_CUT_MIN_CONSECUTIVE_SAMPLES = 3` échantillons consécutifs (garde renforcée par rapport
   aux 2 échantillons habituels, vu la sévérité — Cut vaut 0 point) à l'intérieur du 1/4 NM, même
   zone que le Cut GS existant. Cinq tests de régression ajoutés (`src/grading.rs`) : Cut soutenu
   sink rate, spike isolé non soutenu non cut, hors zone 1/4 NM non cut, Cut soutenu bank angle,
   correction de groove ordinaire (12°) non cut.

Points 3 à 8 sont couverts par de nouveaux tests unitaires dans `src/grading.rs` et `src/track.rs`
(`cargo test --locked` : 187 réussis au moment du point 8) ; aucun n'a de preuve DCS live — le
point 8 encore moins que les autres, puisqu'il n'a explicitement aucune base doctrinale chiffrée à
valider, seulement une cohérence interne (aucun faux positif attendu sur un poser normal). Le
point 1 n'a entraîné aucune modification de code — voir ci-dessus.

Explicitement écarté (pas une piste à reprendre sans nouvelle donnée) :

- Reconstruire une "fenêtre de remise de gaz" dynamique dépendante de la puissance moteur réelle :
  DCS n'expose pas les données nécessaires.
- Entraîner un modèle statistique sur d'anciens vols : aucun corpus de grades LSO humains alignés
  sur les traces DCS du projet n'existe ou n'est raisonnablement constituable.

## Nettoyage technique déjà réalisé (pour référence, ne pas re-proposer)

Les lots suivants, un temps envisagés dans des notes de travail antérieures, sont déjà en place
dans le code actuel : tests autonomes compilables sans fixtures externes, parsing borné du câble
DCS (`1..=4` strict), calculs AoA/vitesse protégés contre les vecteurs dégénérés, registre de
tâches par session/génération avec annulation propre au respawn, migrations SQLite idempotentes
avec propagation des erreurs réelles, rendu PNG déporté en `spawn_blocking`, observabilité
(gaps/skew/latences/percentiles bornés) sans impact sur les décisions métier, `cargo fmt`/`clippy
-D warnings` propres. Toute nouvelle relecture doit repartir du code actuel, pas de cette liste.

## Scénarios de validation Phase 6 (checklist de non-régression avant toute promotion majeure)

- CATOBAR nominal avec câbles DCS 1 à 4 ;
- passage pattern puis finale, sans verrouillage du câble ancien ;
- gaps artificiels de 300 ms, 1 s et livraison retardée ;
- hook timeout/error/stale et passe longue >128 s ;
- simultanéité de plusieurs recoveries et partage éventuel du carrier ;
- reconnect, changement de session et génération ;
- mode sans ACMI ;
- V/STOL séparé sans régression ;
- charge avec détecteurs actifs puis suspendus pendant groove.

## Historique des sessions (résumé, du plus ancien au plus récent)

- **1er septembre 2026** — Refonte v3 : `PositionCollector`/`EventCorrelator`/`ReportPipeline`
  extraits, `--positions-only`, causes multiples, migration SQLite 6, provenance Git/build. Deux
  relectures ("phase 1", "phase 2") ont trouvé puis corrigé : suspension des détecteurs bloquant
  une seconde recovery simultanée, `UnconfirmedArrest` écrasant les causes télémétriques, codes
  gRPC hook mal formatés, `--positions-only` encore couplé à SQLite/Discord, vocabulaire SQLite
  divergent du JSON, indicateur Git dirty incomplet, panne d'événements invalidant à tort la
  télémétrie positionnelle, respawn avec nouvel ID non nettoyé, câbles DCS non bornés à 1–4,
  publication non atomique sur Unix. Toutes ces corrections sont dans le code actuel.
- **4 septembre 2026 (buffer Lua)** — Analyse d'un test local (trois traps `complete/green`, 20 Hz,
  zéro perte) : purge du ring Lua sur `after_sequence` acquitté, `diagnostics` limité à 1×/s,
  `telemetryObservationErrors` bornée à 128 entrées. Côté LSO : sous-échantillonnage des `datums`
  JSON hors fenêtre `scoring_relevant` (1/4), sans toucher à la zone de notation.
- **4 septembre 2026 (notation et cadence)** — Chantier A (A.1 trajectoire continue, A.2 facteur de
  tendance, A.3 pondération temporelle des 150 derniers mètres, A.4 vérification que cause/causes
  séparaient déjà "télémétrie dégradée" du reste, A.5 vent persisté en JSON) et B.2 (commande
  `cadence-ab`) implémentés et validés (fmt/clippy/test/build verts à chaque commit). A.6 (AoA
  réelle via draw argument) non traité à ce stade faute de source fiable.
- **4 septembre 2026 (AoA/vent, commit `108ffa1`)** — A.6 tranché autrement : au lieu du draw
  argument introuvable, l'AoA de `datums`/`pattern_datums` est corrigée du vecteur vent (deux
  appels `AtmosphereService.GetWind` à l'entrée du groove, interpolation par altitude), avec
  `wind_reference_established` comme évidence de repli.
- **4 septembre 2026 (test live CVN-72, 4×F-14 + 4×F-18 IA)** — 14 rapports produits, 0 crash, 0
  erreur télémétrie. Résultats détaillés en tête de ce document (P0/P1/P2 ci-dessus).
- **4 septembre 2026 (ménage documentaire)** — Consolidation de tous les `.md` épars
  (`.agents/agents.md`, `.ignore/*.md`, documents `docs/*.md` obsolètes explicitement marqués comme
  superseded) en trois documents racine : `AGENTS.md`, `tasking-roadmap.md` (ce document) et
  `primer.md`. Les fichiers sources ont été supprimés pour éviter toute divergence future ; leur
  contenu utile est repris ci-dessus.
- **4 septembre 2026 (nouvelles pistes de notation post-test)** — Suite au test live CVN-72,
  identification de quatre pistes supplémentaires non codées pour le calcul de la note (taux de
  descente, détection de surcorrection, angle de gîte, filtre de durée minimale sur l'amplitude
  continue d'A.1) — voir "Pistes d'amélioration de la notation" ci-dessus, points 3 à 6.
- **4 septembre 2026 (correctif remise de gaz en survol)** — Le seul bug P0 confirmé de la session
  précédente (franchissement du seuil de pont sans plafond d'altitude, classant à tort une remise de
  gaz haute en `Bolter`) est corrigé : ajout de `DECK_CROSSING_ALT_CAP_FT` (50 ft, relatif au pont)
  dans [src/track.rs](src/track.rs) et de deux tests de régression (franchissement haut → `WaveoffUnknown`,
  franchissement bas → toujours `Bolter`). `cargo test --locked` (175 réussis), `cargo fmt --check`
  et `cargo clippy --locked --all-targets -- -D warnings` propres au moment du correctif. Non encore
  revalidé sur un enregistrement live — voir "Décisions encore ouvertes".
- **4 septembre 2026 (pistes de notation post-test, notation II)** — Points 3 à 6 de la liste
  "Pistes d'amélioration de la notation" implémentés : garde de persistance A.1
  (`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`), détection de surcorrection A.4/OC
  (`OSCILLATION_MIN_SWING_DEG`/`OSCILLATION_MIN_REVERSALS`), taux de descente et angle de gîte
  portés en contexte sur `TrajectoryDeviation`/`datums` (jamais notés). Point 1 (NC vs statut
  neutre) réexaminé et jugé déjà couvert par `grading_availability` — aucun changement de code.
  Point 2 (AoA cockpit) reste abandonné, inchangé. `src/grading.rs`, `src/track.rs` et
  `src/commands/cadence_ab.rs` modifiés ; `cargo test --locked` (180 réussis), `cargo fmt --check`
  et `cargo clippy --locked --all-targets -- -D warnings` propres. Plusieurs tests existants de
  `src/grading.rs` construits sur un échantillon de trajectoire unique ont dû être adaptés à deux
  échantillons consécutifs pour rester cohérents avec le nouveau garde de persistance. Aucun de ces
  seuils n'a de preuve DCS live — voir "Décisions encore ouvertes".
- **5 septembre 2026 (entrée en groove CATOBAR, roulis/route)** — Deux PDF NATOPS déposés dans
  `.ignore/` (`CV-NATOPS-JUL09.pdf`, `LSO-NATOPS-MAY09.pdf`) comparés au code réel de
  `entered_groove`. Constat : le rayon ¾ NM de la boîte de détection vient en fait du Case III
  (transition au contrôle LSO, 00-80T-104 §6.6.3.1), pas d'un seuil Case I ; le Case I
  (00-80T-105 §6.2.4.2/6.2.4.3) définit le début du groove comme "roll wings level on centerline
  with a centered ball", un événement, jamais une distance/altitude fixe. Ajout de
  `is_rolled_out()` (roulis <= 15° et route sol alignée sur l'axe du groove à <= 15°, confirmés sur
  2 échantillons consécutifs, calculés sur la fenêtre `gate_samples` déjà bufferisée — aucun nouveau
  champ de télémétrie) comme condition supplémentaire, CATOBAR uniquement ; V/STOL Tarawa garde
  l'ancienne boîte seule. Voir point 7 de "Pistes d'amélioration de la notation" ci-dessus pour le
  détail complet. Quatre tests existants de `src/track.rs` (deux sur la trajectoire continue, deux
  sur la distinction bolter/waveoff par franchissement du seuil de pont) construits sur une entrée
  en groove instantanée ont dû être adaptés avec une phase d'approche ailes-à-plat explicite pour
  laisser le temps à la confirmation de se faire ; deux nouvelles assertions vérifient explicitement
  que `entered_groove` est confirmé avant la suite du scénario. `cargo test --locked` (180 réussis,
  0 échec), `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` propres.
  Deux commentaires de code (`entered_groove`, `Grading::WaveoffUnknown`) qui omettaient la
  contrainte de lineup ±10° déjà présente dans le code ont aussi été corrigés au passage. Aucune
  preuve DCS live — voir "Décisions encore ouvertes".
- **5 septembre 2026 (test live CVN-72, 4×F-14 + 4×F-18 IA, Case I)** — Première validation live du
  raffinement CATOBAR de l'entrée en groove livré la même journée : aucune régression détectée sur
  6 appontages et 4 waveoffs, JSON/PNG/ACMI/SQLite systématiquement cohérents entre eux, verdicts
  (`OK`/`(OK)`/`--`/`Cut`/`WO?`) toujours en accord de fond avec le LQM indépendant de DCS malgré des
  mécanismes de calcul totalement différents. Deux bugs P0 confirmés (2/6 appontages chacun) :
  désynchronisation `wire_estimated`/`wire_estimation` et explosion numérique de
  `trajectory_deviations` près du toucher (sans impact sur le verdict dans les cas observés) — tous
  deux corrigés le même jour, voir "Décisions déjà prises" ci-dessus et l'entrée de session
  suivante. Observation opérationnelle : `F18-4-1` a enchaîné 3 waveoffs
  systématiquement coupés à ~165 m, `F18-3-1` n'est jamais descendu sous 100 m sur deux tentatives —
  foul deck suspecté côté mission, hors du périmètre LSO (voir note opérationnelle). Suivi de la
  mémoire du process `DCS_server` démarré en cours de session : légère hausse continue observée
  (+8,3 Mo/6 min) mais sans baseline ni recul suffisant pour conclure. Session arrêtée à la demande
  de l'opérateur après le 3e waveoff de `F18-4-1` ; `lso.exe` arrêté proprement (aucune corruption de
  fichier, l'écriture atomique protège les rapports déjà publiés). Rapports et logs conservés dans
  `.ignore/live-runs/20260905-084848-buffered/`.
- **5 septembre 2026 (correctifs P0 wire_estimated et trajectory_deviations)** — Les deux bugs P0
  confirmés lors du test live CVN-72 du même jour sont corrigés dans `src/track.rs` : (1)
  `cable_estimated` n'est plus figé au moment du toucher/touch-and-go mais toujours réconcilié une
  seule fois dans `Track::finish()` avec `wire_estimation.wire`, contre l'historique complet des
  franchissements de brin ; (2) une nouvelle constante `TRAJECTORY_MIN_DISTANCE_M = 3.0` empêche de
  pousser un échantillon de `trajectory_deviations` sous ce plancher de distance, dans `Track::next`
  et `replay_gate_and_trajectory` identiquement, pour éviter l'explosion `atan2(écart_m, x)` quand
  `x` tend vers 0. Deux tests de régression ajoutés (voir "Décisions déjà prises" ci-dessus)
  rejouent respectivement la course événement/position et l'approche jusqu'au flare réaliste.
  `cargo test --locked --no-fail-fast` (182 réussis, 180 tests du binaire + 2 tests de provenance de
  build), `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` propres.
  Aucune preuve DCS live — les deux enregistrements du 5 septembre où les bugs avaient été observés
  n'ont pas encore été rejoués contre ce correctif, voir "Décisions encore ouvertes".
- **5 septembre 2026 (Cut sink-rate/bank-angle)** — Suite à la question de savoir si le taux de
  descente et l'angle de gîte (pistes 3 et 5 de "Pistes d'amélioration de la notation") influençaient
  la note même indirectement : vérifié que non (grep exhaustif dans `src/grading.rs`, aucune
  occurrence hors valeurs bouchon de test). Avant d'implémenter une influence sur la note, recherche
  documentaire dédiée dans les deux PDF NATOPS (`pdftotext -layout` sur `CV-NATOPS-JUL09.pdf` et
  `LSO-NATOPS-MAY09.pdf`) puis recherche web complémentaire : aucun seuil numérique danger/pas-danger
  codifié pour le sink rate ou le bank angle dans les deux documents (`TMRD`/`W`/`TMA`/`DLW`/`DRW`
  sont des codes de commentaire qualitatifs, laissés au jugement du LSO — même schéma que l'entrée en
  groove Case I). Décision utilisateur explicite malgré cette absence de base doctrinale : Cut direct
  plutôt qu'un simple plafond `(OK)`. Implémenté `dangerous_sink_rate_or_bank()` dans
  `src/grading.rs` : `SINK_RATE_CUT_MPS = 8.0 m/s` (environ le double du régime nominal de poser
  CATOBAR sans flare, ~600-800 ft/min publiquement documenté) ou `BANK_ANGLE_CUT_DEG = 30°` (le
  double de `GROOVE_ROLLOUT_MAX_BANK_DEG`) soutenus sur `DANGER_CUT_MIN_CONSECUTIVE_SAMPLES = 3`
  échantillons consécutifs (garde renforcée par rapport aux 2 échantillons habituels, vu la sévérité
  du Cut) à l'intérieur du 1/4 NM grade désormais la passe `C`, au même titre que le Cut GS existant.
  `persistent_mask()` généralisé pour accepter une longueur de série minimale paramétrable plutôt que
  la seule constante `PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`. Cinq tests de régression ajoutés
  (`src/grading.rs`) : Cut soutenu sink rate, spike isolé non soutenu non cut, hors zone 1/4 NM non
  cut, Cut soutenu bank angle, correction de groove ordinaire (12°) non cut. `docs/GRADING_REFERENCE.md`
  et `AGENTS.md` mis à jour (le sink rate/bank ne sont plus documentés comme "jamais notés"). `cargo
  test --locked --no-fail-fast` (187 réussis, 185 tests du binaire + 2 tests de provenance de build),
  `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` propres. Aucune preuve
  DCS live, et contrairement aux autres seuils de ce module, celui-ci n'a aucun nombre NATOPS à
  recaler dessus — voir "Décisions encore ouvertes" pour le risque de faux positif identifié.
- **5 septembre 2026 (contrôle du Cut sink-rate/bank-angle sur le corpus live existant)** — Sans
  toucher au code, rejeu en JS de la logique exacte de `dangerous_sink_rate_or_bank()` (même filtre
  1/4 NM, mêmes seuils, même garde de persistance à 3 échantillons) contre les 9 rapports JSON de
  `.ignore/live-runs/20260905-084848-buffered/` (6 posés + 3 remises de gaz), pour répondre à la
  question "le nouveau Cut ne serait-il pas trop facile à déclencher, ou pas fiable à cause de
  données manquantes ?" Résultat détaillé ajouté ci-dessus ("Décisions encore ouvertes") : zéro
  déclenchement, zéro cas proche du seuil, `telemetry_quality.completeness: "complete"` et cadence
  20 Hz stable sans irrégularité sur les 9 rapports (aucun signe de donnée manquante/bruitée
  pouvant fausser `sink_rate_mps`). Conclusion : pas trop sévère sur ce corpus, mais ce corpus
  (entièrement IA, approches anormalement lisses) ne peut pas non plus servir à valider que le
  seuil déclencherait au bon moment sur un vrai cas dangereux — la revalidation live reste
  entièrement ouverte. `AGENTS.md`, `primer.md` et `docs/NOTATION-LOGIQUE.md` synchronisés avec
  l'état courant du code (les deux correctifs P0 du 5 septembre et le nouveau Cut sink-rate/bank).
- **5 septembre 2026 (automatisation de `_OK_`)** — Suite à la question "le manuel LSO ne dit-il
  pas que le groove doit durer 15-18 s ?", vérification directe dans le texte extrait de
  `LSO-NATOPS-MAY09.pdf` (déjà `pdftotext`-extrait plus tôt dans la session) : confirmé, NAVAIR
  00-80T-105 §6.2.4.3 le documente littéralement — correction d'une erreur de ma part (j'avais
  écarté ce point comme non fondé lors de la proposition initiale du design). Design finalisé avec
  l'utilisateur en trois échanges : garde anti-bruit appliquée à `_OK_` comme partout ailleurs,
  fenêtre de temps NATOPS 15-18 s appliquée à tous les CATOBAR malgré la variation de pente du
  T-45. Implémenté dans `src/grading.rs` (`is_amplitude_perfect`, nouvelle branche de
  `grade_from_gates` prenant `groove_time_secs`, exclusion explicite du touch-and-go) — détail
  complet dans "Décisions déjà prises" ci-dessus. Deux tests existants, qui verrouillaient l'ancien
  comportement "jamais de `_OK_` automatique", réécrits pour vérifier le nouveau comportement
  intentionnel plutôt que supprimés silencieusement. Sept nouveaux tests de régression couvrent
  l'indépendance au brin, les deux bornes de la fenêtre de temps (inclusive/exclusive), l'exclusion
  du touch-and-go, une porte hors bande, et le pardon/refus du pic de trajectoire isolé vs soutenu.
  `cargo test --locked --no-fail-fast` (193 réussis, 191 tests du binaire + 2 tests de provenance
  de build), `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` propres.
  `AGENTS.md`, `primer.md` et `docs/NOTATION-LOGIQUE.md` mis à jour en respectant l'objet de
  chacun (index machine-first, vulgarisation complète du pipeline, résumé pédagogique de la seule
  notation). Aucune preuve DCS live — voir "Décisions encore ouvertes".
- **5 septembre 2026 (calibration de crosse T-45/F-14)** — Suite à une question sur la fiabilité
  de la distinction touch-and-go/bolter (réponse : fiable dans son principe mais limitée au
  F/A-18C seul, faute de calibration pour les autres types), l'utilisateur a fourni les index de
  draw argument manquants : 25 pour le T-45 (identique au F/A-18C), 1305 pour le F-14 (toutes
  variantes). Implémenté `AirplaneInfo::hook_draw_argument` (`src/data.rs`) et généralisé
  `Track::calibrated_hook_state` (`src/track.rs`) pour se baser sur la présence de cet index plutôt
  que sur un nom de type codé en dur ; les deux points d'appel gRPC
  (`src/tasks/record_recovery.rs`) interrogent désormais l'index propre à chaque type. Détail
  complet, limite assumée (polarité non confirmée pour T-45/F-14) et tests réécrits/ajoutés dans
  "Décisions déjà prises" ci-dessus. `cargo test --locked --no-fail-fast` (194 réussis, 192 tests
  du binaire + 2 tests de provenance de build), `cargo fmt --check` et `cargo clippy --locked
  --all-targets -- -D warnings` propres. `AGENTS.md`, `docs/GRADING_REFERENCE.md` et `primer.md`
  mis à jour ; `docs/NOTATION-LOGIQUE.md` non touché (hors de son périmètre déclaré : détection
  d'issue, pas logique de notation). Aucune preuve DCS live — voir "Décisions encore ouvertes".
- **5 septembre 2026 (test live CVN-72, 4×F-14 + 4×F-18 IA, session du soir)** — Rebuild release
  après les modifications non committées de la journée (`src/track.rs`, `src/grading.rs`,
  `src/data.rs`, `src/tasks/record_recovery.rs`), puis lancement de
  `.ignore/run-live-buffered.ps1` avec mission en pause, retirée par l'utilisateur une fois le
  module confirmé connecté (`session_id=1788610239`, `generation=9`). Session arrêtée par
  l'utilisateur après 5 appontages complets (avant les 3 F/A-18 restants) pour passer aux
  conclusions. Aucun crash, aucune erreur télémétrie sur toute la session ; JSON/PNG/ACMI cohérents
  à chaque appontage. Résultats par avion :
  - `F14-1-1` (brin 2) : trois portes propres (<=0,55°), brin Rust/DCS en accord (2/2), mais note
    finale `NoGrade` (`--`) à cause de l'écart continu grimpant à 10,6° GS / -10,2° lineup à 4 m du
    pont (flare géométrique, voir P0 ci-dessus).
  - `F14-2-1` (brin 3) : trois portes excellentes (<=0,60°), DCS a lui-même noté `GRADE:_OK_`, brin
    Rust/DCS en accord (3/3) — mais note finale projet retombée à `NoGrade` pour la même raison
    géométrique (10° GS / -7,2° lineup à 6 m). Première confirmation empirique que ce défaut
    affecte même une passe validée `_OK_` par le jeu.
  - `F14-3-1` (brin 1) : Cut, GS réellement bas dans le ¼ NM, cohérent avec le `GRADE:---` de DCS —
    au départ noté "pas de bug", mais une reconstruction ultérieure de l'écart réel en mètres (voir
    le correctif `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` ci-dessous) a montré que ce Cut précis était
    lui aussi partiellement artefactuel : un biais bas réel d'environ 1,4-1,7 m, stable voire en
    légère amélioration depuis ~110 m, n'a franchi le seuil angulaire -2,5° qu'une fois `x` tombé
    sous ~31 m. Le pilotage réel était mauvais (biais réel important) mais pas nécessairement
    digne d'un Cut à 0 point plutôt que d'un `--` ; corrigé le jour même.
  - `F14-4-1` : Cut légitime (GS bas, -2,56° puis -3,36°), cohérent avec DCS. Première divergence
    de brin observée : Rust=1 vs DCS=2, `wire_primary: dcs_lqm` a correctement protégé la valeur
    pilote-facing (voir P1 ci-dessus).
  - `F18-2-1` (premier F/A-18 Hornet du test) : Cut légitime (GS bas puis flare cumulé, -2,8° à
    6,8 m puis -7,0° à 3,7 m), cohérent avec DCS. Deuxième divergence de brin : Rust=2 vs DCS=3,
    même garde-fou correct.
  Trois nouveaux points ouverts identifiés et documentés ailleurs dans ce fichier : l'écrasement de
  note par la géométrie de flare près du toucher (P0, à corriger avec l'utilisateur), le possible
  biais de -1 brin sur l'estimation Rust (P1, 2 échantillons seulement), et l'absence de
  `groove_time_secs` dans le JSON (corrigé le jour même, voir "Décisions déjà prises"). Pas de
  répétition du WARN `duplicate touchdown ignored` en anomalie : confirmé normal à chaque
  appontage (`events[]` montre le second événement `land` explicitement rejeté par corrélation
  géométrique/ID, sans impact sur le résultat).
- **5 septembre 2026 (correctif de l'amplification géométrique près du toucher)** — Suite au P0
  identifié lors du test live du soir même, conception du correctif directement à partir des 5
  rapports JSON déjà produits : reconstruction de l'écart réel en mètres à chaque échantillon
  (`distance_m * tan(gs_deviation_deg)`/`tan(lineup_deg)`) sur les 12 derniers points de chaque
  appontage, révélant que l'écart vertical/latéral sous-jacent était en fait quasi constant
  (~0,75-1,1 m) plutôt que croissant — confirmant que l'angle grimpant à deux chiffres était un pur
  effet du dénominateur `x` qui rétrécit, pas un vrai comportement de pilotage qui s'aggrave. La
  même reconstruction sur les trois Cuts a montré que l'un d'eux (F14-3-1) était partiellement
  artefactuel de la même façon (biais réel stable ~1,4-1,7 m, Cut déclenché seulement une fois `x`
  petit) — note de session du test live corrigée en conséquence ci-dessus. Décision utilisateur
  explicite sur l'approche : la zone doit continuer à pouvoir dégrader la note comme avant (pas de
  simple arrêt d'enregistrement), et une nouvelle façon de calculer était préférée à un correctif
  ad hoc, avec les choix de conception laissés à l'appréciation de l'agent.

  Conception retenue : substituer une distance de référence fixe à `x` dans le calcul d'angle une
  fois `x` sous ce seuil, plutôt qu'un système parallèle en mètres avec ses propres seuils — réutilise
  telle quelle toute la logique en aval (grille de note, Cut, `_OK_`) sans la dupliquer, et reste
  continue à la frontière du seuil (aucune discontinuité au moment où `x` la franchit). Seuil
  `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75 m` choisi par cohérence physique (~1 s de vol à ~75 m/s en
  approche CATOBAR), vérifié à la main contre les 5 rapports du soir pour confirmer qu'il replace
  bien les deux passes propres en dehors de `NoGrade` tout en laissant un écart réellement important
  (`atan2(-3,5 m, 75 m) ≈ -2,67°`) toujours capable de déclencher un Cut. Implémenté dans
  `trajectory_deviation_angles_deg`, nouvelle fonction partagée par `Track::next` et
  `replay_gate_and_trajectory` (`src/track.rs`) — même précaution de point de calcul unique que pour
  l'ancien bug `wire_estimated`/`wire_estimation`, pour ne jamais laisser les deux chemins diverger.
  Deux tests de régression ajoutés : un écart constant de 0,8 m de 50 m à 4 m reste sous 1°/2° (au
  lieu d'exploser à deux chiffres), un écart réel de 3,5 m déclenche toujours le Cut. `cargo test
  --locked --no-fail-fast` (196 réussis, 194 tests du binaire + 2 tests de provenance de build),
  `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` propres. En creusant
  la reconstruction des écarts, repéré un effet secondaire distinct non résolu par ce correctif :
  l'écart latéral quasi constant lui-même (~0,75-0,85 m, présent à toute distance sur les 5
  appontages, propres comme Cut) ne ressemble pas à une vraie erreur d'alignement croissante — piste
  ajoutée en P1 pour une session future. `AGENTS.md`, `docs/GRADING_REFERENCE.md` et
  `docs/NOTATION-LOGIQUE.md` mis à jour. Aucune preuve DCS live sur ce correctif précis — voir
  "Décisions encore ouvertes".
