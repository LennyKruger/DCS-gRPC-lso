# Tasking & roadmap — DCS-gRPC-lso

> Idées, choix techniques ouverts, arbitrages à faire et bugs connus non résolus ou non revalidés.
> Ne contient que des points **encore en suspens** — dès qu'un point est résolu et revalidé (ou
> tranché sans besoin de revalidation), il est retiré d'ici ; l'essentiel synthétique de ce qui a
> été fait migre vers [CHANGES.md](CHANGES.md) (et [AGENTS.md](AGENTS.md) si ça touche l'état
> durable du système) — voir [AGENTS.md](AGENTS.md), "Règles de maintenance des documents markdown
> racine", pour la règle complète. Pour le détail narratif des sessions passées (dates, tests,
> discussions de conception), `git log`/`git show` sur les commits correspondants fait foi ; ce
> document ne le duplique plus. Dernière purge/fusion : 6 septembre 2026, intégrant l'analyse d'un
> second test live CVN-72 du 5 septembre 2026 (soirée, pilotes humains `Ghost-72 | TT` et
> `Justice`, 8 recoveries T-45/F-14B(U), commit `96fd52e` dirty), puis les trois correctifs P0 qui
> en découlent, un changement de règle discuté et validé avec l'utilisateur (porte 3/4 NM plus
> exigée pour `_OK_`/`OK`/`(OK)` quand elle précède l'entrée en groove), et une amélioration de
> précision de la détection de roll-out elle-même (régression linéaire remplaçant la comparaison à
> deux points), tous implémentés et testés unitairement le 6 septembre 2026 (working tree non
> committé au moment de cette mise à jour — voir
> [AGENTS.md](AGENTS.md) en tête de document).

## À faire en priorité (P0)

Aucun point P0 ouvert. Les trois bugs de classification confirmés lors du test live CVN-72 du 5
septembre 2026 (soir, pilotes humains) ont été corrigés le 6 septembre 2026 — `cargo test`
(200 réussis, 0 échec, dont 4 nouveaux tests dédiés), `cargo fmt --check` et
`cargo clippy --locked --all-targets -- -D warnings` propres ; voir [CHANGES.md](CHANGES.md)
(`Unreleased`) pour le détail de chaque correctif et [AGENTS.md](AGENTS.md), "Gates, outcomes et
câble", pour l'état factuel courant. Aucun des trois n'a encore de preuve DCS live confirmant
l'absence d'effet de bord — voir les trois entrées correspondantes dans "Décisions encore ouvertes"
ci-dessous.

## P1 — bugs confirmés à corriger, décisions à prendre

- **Estimation Rust du brin systématiquement décalée par rapport à l'événement DCS.** Précédemment
  listé ici comme « possible biais de -1 brin » sur 2 échantillons (F14-4-1, F18-2-1) — le test du 5
  septembre soir apporte un 3e cas et un **mécanisme identifié** : sur le rapport où Rust (brin 4,
  confiance `medium`) diverge de DCS (brin 1), le `runway_touch` DCS arrive après les 4
  franchissements géométriques successifs ; l'estimateur (`continuous_hook_plane_crossing`) retient
  le **dernier brin franchi avant l'événement DCS**, donc systématiquement le plus haut numéroté
  quand l'événement DCS est en retard — cohérent avec un biais dans le même sens sur les 2 cas
  précédents. Deux traps du même test montrent par ailleurs `wire_crossing_not_time_correlated_with_
  event` (aucune estimation) quand le `runway_touch` arrive plus d'1 s après le franchissement
  géométrique, et deux survols/bolters sans accrochage produisent une estimation « high » sémantique-
  ment fausse (non exposée au pilote, `wire_primary: dcs_lqm` fonctionne correctement dans tous les
  cas observés). Probablement aggravé par l'offset vertical de crosse F-14 (voir "Décisions encore
  ouvertes" ci-dessous, corrigé le 6 septembre 2026 mais non revalidé en mission live). Piste :
  corréler non pas sur l'horodatage de l'événement DCS mais sur le début de la décélération
  (`touchdown_horizontal_speed_mps`) ou le premier contact géométrique, et ne jamais produire de
  confiance « high » sans preuve d'arrêt. À rejouer avec `cadence-ab` une fois l'offset corrigé pour
  voir si le biais de -1 brin disparaît.
- **`baseline_manifest` sérialisé entièrement `null` dans le JSON malgré un manifeste valide fourni
  au démarrage.** Le manifeste (6 clés, aucune erreur de validation) est accepté puis perdu entre
  `run.rs` et `RecoveryReport`, ou le champ du rapport est alimenté depuis une source jamais remplie.
  La provenance de build (`lso_commit`/`lso_dirty`/`dcs_grpc_version`), elle, est correcte. À noter
  aussi : un manifeste chargé une fois au démarrage ne peut pas suivre un changement de mission en
  cours de session (observé le 5 septembre : `_A` → `_B` après coup) — envisager d'horodater la
  lecture du manifeste dans le JSON, ou de le recharger à chaque nouvelle session DCS.
- **Référence de vent incohérente : `180°/0,0 m/s` sur 2 rapports sur 8**, contre `95°/0,99-1,42 m/s`
  cohérent sur les 6 autres (même navire, même mission, à quelques minutes d'écart). Remplace/précise
  l'ancien point P2 « vent nul non exercé en test » : le vent réel **a bien été exercé** cette fois,
  et la mécanique révèle une anomalie plutôt qu'une simple absence de couverture de test — `180/0`
  ressemble à une valeur par défaut ou à un `GetWind` ayant renvoyé un vecteur nul sans faire
  retomber `wind_reference_established` à `false`. Conséquence : l'AoA corrigée retombe silencieuse-
  ment sur l'approximation brute. À investiguer côté séquence des deux appels
  `AtmosphereService.GetWind`/interpolation par altitude (`src/track.rs`) : un des deux appels a-t-il
  échoué silencieusement ?
- **Fenêtre de temps de groove `_OK_` (15-18 s NATOPS) jamais atteinte par un pilote humain sur ce
  test** : `groove_time_secs` mesuré à 19,8 / 20,3 / 22,4 / 23,0 / 26,1 s sur les 5 passes où il est
  connu, y compris sur le F-14 le plus rapide (138 kt, 19,8 s). Le temps est compté depuis le
  roll-out confirmé (870-1 384 m) : à 60-70 m/s, 1 200 m représentent déjà 17-20 s. Aggrave l'item
  déjà ouvert plus bas (« Cut sink-rate/bank ») sur `_OK_` : soit le roll-out humain typique est plus
  large/précoce qu'un pattern NATOPS de référence (abeam 1,2-1,3 NM viserait plutôt 15-18 s), soit la
  borne haute mérite d'être revue par type, **soit ce corpus précis reflète simplement un pilotage
  humain perfectible plutôt qu'un problème de calibration** (remarque de l'utilisateur, 6 septembre
  2026 : les deux pilotes de ce test n'étaient pas d'un niveau irréprochable) — donnée encore
  insuffisante (5 échantillons, tous IA de la veille exclus, aucun repère de niveau de pilotage) pour
  trancher entre ces trois causes. Le signal reste cohérent et pousse à traiter cette question avant
  toute promotion de `_OK_` comme fiable en usage humain, mais sans présumer aujourd'hui laquelle des
  trois explications est la bonne.
- **Observabilité manquante confirmée sur ce test** (aucune ligne au niveau par défaut ne permet de
  diagnostiquer trois situations vécues ce soir) : (1) deux générations de superviseur consommées
  sans une seule ligne INFO/WARN pendant un rechargement de mission (à côté, côté DCS,
  `grpc.lua:288: attempt to index global 'grpc' (a nil value)` pendant ce même rechargement, sans
  suite après reconnexion 14 s plus tard) — logger la raison de fin de génération et au moins la
  première tentative de reconnexion échouée ; (2) aucune trace d'envoi Discord réussi/échoué ni
  d'insertion SQLite au-delà d'une ligne « integration enabled » au démarrage — une ligne INFO par
  publication réglerait ça pour un coût de 2 lignes/rapport ; (3) motif d'abandon d'une tentative de
  détection uniquement en DEBUG (voir aussi le point P2 correspondant ci-dessous, déjà listé, avec
  cette fois des chiffres concrets : 3 faux départs sur la soirée, 2 à 55 s/2 s de flux, un à 52 s/8
  s, un à 42 s, dont l'un a couvert le catapultage T-45 lui-même).

### Hypothèses non confirmées, à investiguer plus avant

- **Écart latéral quasi constant (~0,75-0,85 m) présent à toute distance**, observé sur les 5
  appontages d'un premier test live CVN-72 du 5 septembre 2026 (matin), réussis comme Cut. Repéré en
  reconstituant l'écart réel en mètres depuis `lineup_deg`/`distance_m` du JSON : l'écart latéral ne
  grossit pas proportionnellement à la distance (ce qu'on attendrait d'une vraie erreur d'alignement
  ou d'un axe de référence mal orienté) — il reste quasiment constant de 50 m jusqu'au toucher, sur
  les 5 appontages sans exception. Ressemble davantage à un décalage fixe de point de référence
  (crosse vs CG projetée, ou point de visée supposé) qu'à un vrai écart d'alignement croissant. **À
  distinguer du biais vertical de crosse F-14B(U) déjà corrigé** (voir "Décisions encore ouvertes"
  ci-dessous, `F14_HOOK_VERTICAL_CORRECTION_M`) : celui-ci porte sur l'altitude, pas le lineup, mais
  la parenté (même famille de cause : offset de crosse mal placé dans le repère avion) mérite d'être
  vérifiée maintenant que le correctif d'offset vertical est livré — un offset latéral mal calibré
  au même endroit du code n'est pas à exclure. N'affecte pas la note de façon significative
  (`NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` le neutralise déjà en grande partie) et n'a donc aucune
  urgence propre.
- **Déclin de l'AoA corrigée dans le groove** (~1,5-3° entre ¾ NM et ¼ NM, systématique sur 3-4 F-14
  dans un test avec vent nul, matin du 5 septembre). Le vent nul dans cette mission exclut un
  artefact de la correction vent introduite pour l'AoA ; reste à savoir si c'est un comportement de
  pilotage IA réel ou un effet de la décomposition en repère avion. **Distinct** de l'aberration AoA
  à hauteur de pont observée le soir même (chute brute type 9,8°→0,8° en 0,6 s à l'arrondi, sur une
  remise de gaz — approximation géométrique qui se dégrade dès que l'avion touche/frôle le pont,
  jamais notée mais affichée, contexte uniquement) : les deux méritent un outil de comparaison
  ancienne/nouvelle formule sur les mêmes données, mais ne semblent pas être le même phénomène (l'un
  progressif sur toute l'approche, l'autre un artefact ponctuel près du pont).
- **Délai fixe touchdown → fichiers complets.** Reconfirmé le 5 septembre soir à **10,1 s** (contre
  ~10,2 s le matin) pour les 4 appontages accrochés du test, et **2,2 s** pour les 4 T&G/bolters/
  survols du même test — l'écart entre les deux confirme qu'il ne s'agit pas d'un temps de calcul
  fixe mais d'un délai propre au chemin `Arrested` : la timeline crosse s'arrête ~6 s après le
  toucher physique (relevage de crosse, arrêt de l'avion), suivi d'environ 4 s de finalisation
  (rendu PNG mesuré à 38-78 ms, donc négligeable dans ce total). Le mécanisme est donc mieux compris
  qu'avant, mais sa source exacte dans le code (confirmation finale du brin ? cadence de lecture du
  buffer côté fork ?) reste à confirmer directement dans `src/tasks/record_recovery.rs`.
- **Mémoire du process `DCS_server` en légère hausse continue.** Le test du 5 septembre soir mesure
  +700 Mo de working set sur 63 min, mais sur une mission Foothold dynamique avec combats et 2-3
  joueurs — **non comparable** au +8,3 Mo/6 min observé le matin sur une mission minimaliste (charge
  et durée trop différentes pour confirmer ou infirmer la même hypothèse). LSO n'écrit rien côté DCS
  et ne peut qu'observer depuis l'extérieur (compteur Windows du process) ; toujours à refaire sur
  une session dédiée, plus longue, à charge de mission constante et connue, avec mesure dès le
  démarrage du serveur.

## Optimisations et robustesse à considérer (P2)

- **Boucle de redémarrage pendant une pause/un rechargement de mission** — reconfirmé le 5 septembre
  soir : deux générations consommées sans log exploitable, puis deux générations de repli
  (`session_id` négatif) créées à 6 s d'intervalle sur `GetSessionId` en timeout jusqu'à
  rechargement complet, en cohérence avec l'erreur Lua `grpc.lua:288` côté DCS pendant le
  rechargement (voir aussi le point d'observabilité en P1 ci-dessus). Un backoff progressif
  réduirait le bruit de logs lors d'une pause prolongée.
- **Purge du ring Lua sur `after_sequence` acquitté** et **`telemetryObservationErrors` bornée à 128
  entrées** : implémentées côté fork. Le test du 5 septembre soir apporte un signal négatif à
  surveiller (pas une régression confirmée) : `recovery_telemetry.overflow_count` correspond
  exactement à `snapshots_received - 600` sur les 8 rapports, et `high_water_mark` reste à `600/600`
  en permanence — le ring semble rester plein et évincer par capacité plutôt que par purge sur
  acquittement, ou alors le compteur comptabilise les évictions post-acquittement comme des
  débordements. Aucune perte réelle ce soir (`lost_snapshots: 0`), mais la métrique est aujourd'hui
  inexploitable telle quelle. Côté fork, ne pas modifier sans demande explicite — mais à
  instrumenter avant de conclure.
- **Tentatives d'approche avortées avant le groove, invisibles hors logs DEBUG.** Déjà listé ;
  reconfirmé avec des chiffres concrets le 5 septembre soir : 3 faux départs de détection sur la
  soirée (un au catapultage T-45 lui-même : 55 s puis 2 s de flux bufferisé + échantillonnage crosse
  ouverts puis arrêtés sans motif visible ; deux autres sur un F-14, 52 s/8 s puis 42 s), pour un
  coût mesuré d'environ 2,5 min de flux inutiles et 5 cycles start/stop côté Lua sur la soirée.
  Piste additionnelle par rapport à la version précédente de ce point : exclure de l'armement du
  détecteur un avion qui vient de se trouver sur le pont (position proche de zéro, altitude proche de
  zéro, vitesse < 30 m/s) pendant une fenêtre de N secondes, ou exiger une vitesse verticale négative
  et une distance déjà positive pour armer `detect_recovery_attempt` ; et logger le motif d'abandon
  en INFO plutôt qu'en DEBUG.
- **`hook_history_truncated` remonté comme cause secondaire sur 6 rapports sur 8** dans le test du 5
  septembre soir, alors que la fenêtre finale de notation (35 à 79 échantillons) reste toujours
  intacte : la timeline crosse est plafonnée à 512 entrées (`MAX_HOOK_EVIDENCE`), et un pattern
  humain dure 2,5-3 min entre détection et toucher (4 Hz × 180 s = 720 échantillons attendus), donc
  la troncature des échantillons **pré-groove** est structurelle pour un pilote humain — elle ne
  retire jamais d'évidence utile à la notation, mais pollue `causes` sans raison. Piste : compacter
  les échantillons pré-groove (ne garder que les transitions, comme le fait une autre lignée
  antérieure du programme) plutôt que les tronquer par FIFO, ou ne plus remonter la troncature dans
  `causes` quand `samples_in_final_window` est déjà complet.
- **Incohérence `datums[].alt` (clampé à 0) vs `trajectory_deviations[].alt_m` (non clampé)** :
  observé à la même position/au même instant dans plusieurs rapports du 5 septembre soir (ex. `alt =
  0.00` vs `alt_m = -0.8` pour la même approche), ce qui masque justement l'information « crosse sous
  le pont » qui a permis de diagnostiquer le biais vertical de crosse F-14 (voir "Décisions encore
  ouvertes" ci-dessous). Deux définitions différentes d'une même grandeur dans un même rapport ; à
  uniformiser (ne plus clamper l'une, ou clamper les deux de façon cohérente) maintenant que le
  biais de crosse lui-même est corrigé.
- **Nommage `aircraft_id` trompeur** : vaut un index de type (0 pour T-45, 3 pour F-14B(U)), pas un
  ID d'unité, dans un rapport qui porte par ailleurs `carrier_id` = vrai ID d'unité — source de
  confusion pour un consommateur externe du JSON.
- **Taille des JSON de rapport (0,94 à 1,50 Mo)** : ~90 % provient de `datums` (883-1 566 points
  malgré le sous-échantillonnage 1/4 pré-groove) et de `hook_observation.timeline` (512 entrées, dont
  ~430 pré-groove sans intérêt une fois la fenêtre finale connue). Compacter la timeline crosse hors
  groove (même piste que `hook_history_truncated` ci-dessus) diviserait la taille par ~2 sans perte
  d'évidence utile.
- **Script `run-live-buffered.ps1` non autonome pour un déploiement hors du poste de développement**
  (observé le 5 septembre soir, sur la machine de test qui n'a pas le lecteur `E:` du dépôt de
  développement) : exige `--baseline-manifest live-baseline.json` dans son propre dossier sans que ce
  fichier soit déployé avec le script (le rendre optionnel si absent, ou documenter qu'il doit être
  livré à côté) ; variable `$lsoRoot` calculée et jamais utilisée ; webhook Discord en clair dans le
  script **et** dans `start.bat` (à externaliser en variable d'environnement, comme pour le token
  gRPC). Par ailleurs, le dossier `Records` de cette machine mélange des rapports `schema_version: 3`
  / `project-derived-v4` (0.2.0, cette lignée) et des rapports `schema_version: 8` / `lso_version
  0.4.0` / `project-derived-v1` d'une **autre lignée** du programme, tous deux écrits dans le même
  `lso.db` — aucune erreur SQLite observée ce soir (WAL passé de 123 à 194 Ko sur 8 rapports, donc les
  insertions passent), mais la cohabitation des migrations 2-6 de cette lignée avec les
  tables/migrations de l'autre lignée doit être vérifiée directement avec `sqlite3` avant de faire
  confiance au greenie board sur une base partagée entre les deux lignées.
- **Isolation multi-navires confirmée, simultanéité toujours non testée.** Le test du 5 septembre
  soir avait trois porte-avions dans la mission (CVN-72, CVN-74, Tarawa) ; seul le CVN-72 a été
  approché, sans aucune détection parasite sur les deux autres — confirme l'isolation par
  paire/navire. Ne change rien à l'item déjà listé plus bas dans "Hors-scope confirmé" : les 8
  recoveries de ce test sont restées séquentielles, jamais deux avions en approche simultanée sur le
  même navire.

## Hors-scope confirmé (rappel volontaire)

- **Robustesse multi-recoveries simultanées** : ni la première mission de test (8 unités) ni le test
  du 5 septembre soir (recoveries séquentielles malgré 3 porte-avions dans la mission) n'ont permis
  de tester deux avions en approche en même temps.
- **Cadence adaptative pré-groove (100/200 ms)** : toujours en attente d'une décision explicite,
  indépendante de tout test. `lso.exe cadence-ab` (voir [AGENTS.md](AGENTS.md)) est l'outil de
  mesure prévu pour instruire cette décision, pas la décision elle-même. Exécuté une fois sur un
  corpus de 9 rapports live : aucun changement de grade, mais 2 occurrences où la porte ¾ NM devient
  invalide sur l'enregistrement le plus récent, déjà sous-échantillonné côté capture — confirme
  empiriquement le risque déjà identifié (un sous-échantillonnage supplémentaire cumulé casse la
  porte la plus proche du groove).
- **Nouveau message `RecoveryTelemetry` compact** : changement de protocole additif, deux dépôts,
  régénération des stubs — chantier séparé, non entamé.

## Note opérationnelle (procédure de test, pas un bug LSO)

Arrêter une tâche de surveillance qui encapsule `lso.exe` dans son propre pipeline tue aussi
`lso.exe` — a coûté l'enregistrement d'un trap en cours d'approche lors d'un test. À éviter dans une
future session : lancer le process et le monitoring séparément.

**Foul deck suspecté côté mission, pas côté LSO (5 septembre 2026, matin).** En fin de session, un
avion a enchaîné 3 waveoffs consécutifs coupés systématiquement à ~165 m du pont (`WO?`, DCS
`GRADE:WO WOFDIC` à chaque fois), et un autre n'est jamais descendu sous 100 m MSL sur ses deux
tentatives. Schéma cohérent avec un pont indisponible (gear pas en batterie, zone d'appontage
occupée) plutôt qu'avec un problème de pilotage IA répété. DCS-gRPC n'expose aujourd'hui aucun signal
d'état du pont/de la passerelle arrière ; LSO ne peut donc ni confirmer ni infirmer cette hypothèse et
documente fidèlement "remise de gaz, initiateur inconnu" à chaque fois, comme prévu par conception
(voir [AGENTS.md](AGENTS.md), "jamais inventer OWO/WOP/waveoff pilote"). À vérifier directement dans
la mission/le jeu lors d'une prochaine session, pas par une modification de LSO. Le test du soir du
même jour n'a montré aucun signe équivalent (les remises de gaz observées ont des causes distinctes,
voir le bug `Bolter` sans preuve de contact plus haut).

## Décisions encore ouvertes (mission/serveur nécessaires)

Tous les points ci-dessous sont corrigés et testés unitairement (`cargo test`/`clippy`/`fmt`
propres au moment de leur implémentation, voir [CHANGES.md](CHANGES.md) pour le détail de chaque
changement), mais la preuve DCS live disponible reste partielle :

- **Porte 3/4 NM plus exigée pour `_OK_`/`OK`/`(OK)` CATOBAR quand elle précède l'entrée en
  groove confirmée** (implémenté le 6 septembre 2026, décision de conception plutôt qu'un bug —
  discutée et validée avec l'utilisateur, qui a confirmé vouloir se limiter à ignorer
  conditionnellement cette seule porte plutôt que retirer toute exigence de gate). Sur un pattern
  Case I humain, la porte 3/4 NM tombe structurellement dans le virage base-à-finale plutôt que
  dans le groove (confirmé live 5 septembre 2026 soir : 7 passes sur 8, lineup jusqu'à -10,5° à
  cette porte), imposant `--` indépendamment du groove réellement volé ensuite. La porte (présente
  ou non, valide ou non) ne compte plus ni pour l'éligibilité ni pour l'amplitude GS/lineup retenue
  lorsque son timestamp précède `groove_entry_time` — seules les portes 1/2 NM et 1/4 NM (plus la
  trajectoire continue) restent exigées dans ce cas ; une porte 3/4 NM capturée après l'entrée en
  groove reste exigée comme avant. Ne touche ni V/STOL (garde la règle inconditionnelle, faute
  d'entrée en groove confirmée par roll-out à laquelle ancrer la relaxation) ni `lso.exe
  cadence-ab` (ne rejoue jamais l'entrée en groove, donc toujours `None`). Aucune source `OFFICIAL`
  ne fait d'un franchissement de porte fixe une condition de qualification (voir AGENTS.md,
  "Gates, outcomes et câble") ; cette relaxation ne fait donc que retirer une exigence
  `PROJECT-DERIVED` non doctrinale, elle n'assouplit aucune règle NATOPS. Reste entièrement à
  revalider en mission live : (1) rejouer avec `cadence-ab` (une fois cet outil mis à jour pour
  transmettre une entrée en groove reconstituée, ou par une revalidation manuelle) les 8 rapports
  du 5 septembre soir pour confirmer que les passes autrement propres remontent bien à `OK`/`(OK)`
  et quantifier combien de `--` étaient artefactuels ; (2) vérifier que la relaxation ne masque
  jamais un vrai défaut d'alignement en base/début de virage qu'un LSO humain aurait commenté
  (`LULR`/`LURC` etc.) — la porte 3/4 NM reste conservée et diagnostiquée dans le rapport, seulement
  non comptée, donc aucune donnée n'est perdue, mais l'absence de sanction sur un lineup réellement
  dangereux mérite un second regard humain sur un corpus plus large.
- **`Bolter` sans preuve de contact, et refus d'un `Bolter` que le LQM DCS contredit** (corrigé le
  6 septembre 2026, ex-P0). Un `Bolter` déduit de la seule géométrie (aucun événement `Land`/
  `RunwayTouch` jamais corrélé) exige désormais un contact confirmé
  (`deck_crossing_confirmed_contact`, hauteur de crosse au franchissement
  `<= DECK_CONTACT_CONFIRMATION_ALT_M = 1,0 m`) ; sinon `WaveoffUnknown`. Séparément, `Track::finish`
  refuse tout `Bolter` (géométrique ou événementiel) si le LQM DCS ouvre sur `GRADE:WO`
  (`dcs_grade_is_waveoff`). Corrige les deux cas confirmés le 5 septembre (survol à 8,7 m ; LQM
  `GRADE:WO ... WO(AFU)IC` contredisant un `Bolter` géométrique). Reste entièrement à revalider en
  mission live : (1) que le seuil de 1,0 m (choisi pour tolérer le bruit de houle/télémétrie autour
  d'un vrai contact, sans base chiffrée) ne fait manquer aucun vrai bolter à contact bref/léger ;
  (2) que le filtre `GRADE:WO` ne masque jamais un vrai bolter que DCS aurait par ailleurs
  correctement noté avec un commentaire commençant malgré tout par `WO` pour une autre raison. Le
  troisième cas du test (rapport 5, hook réellement au contact via un `runway_touch` DCS avant un
  rebond, Own Wave Off au dire du pilote) n'est **pas** couvert par ce correctif : il reste classé
  `Bolter` par conception, faute de moyen de distinguer un contact crosse seul d'un vrai contact
  roues côté DCS — voir la piste de signature sink-rate/LQM plus haut si une distinction OWO
  documentée `PROJECT-DERIVED` est un jour décidée.
- **Offset vertical du point de crosse F-14 (`F14_HOOK_VERTICAL_CORRECTION_M = +1,0 m`, `src/
  data.rs`)** (corrigé le 6 septembre 2026, ex-P0). Correction empirique ajoutée par-dessus la
  position ModelViewer2 brute (`F14_HOOK_MODEL`), qui plaçait le point de crosse ~0,8-1,1 m sous le
  niveau réel du pont pendant qu'un F-14B(U) y roulait. **Confirmée live pour le F-14B(U)
  uniquement** ; reste à faire : (1) vérifier sur F-14A/F-14B (même constante, jamais testée
  séparément) dès qu'un enregistrement humain existe pour ces variantes ; (2) obtenir une vraie
  remesure ModelViewer2 pour remplacer cette correction empirique par une position exacte ; (3)
  rejouer avec `cadence-ab` les rapports F-14 affectés du 5 septembre (bolters et traps) pour
  quantifier combien de Cut/`--` près du pont étaient artefactuels ; (4) vérifier si le biais de -1
  brin de l'estimation Rust (voir P1 plus haut) disparaît une fois ce correctif en place.
- **Lecture de crosse figée au premier contact géométrique plutôt qu'à l'événement DCS**
  (`first_hook_ground_contact_time`, corrigé le 6 septembre 2026, ex-P0). `calibrated_hook_state` ne
  retient plus que les échantillons antérieurs au premier instant où la hauteur de crosse calculée
  atteint `<= 0`, au lieu de l'ancien repère `landing_time` (événement DCS, en retard de ~0,2-1 s sur
  le contact physique confirmé live). Corrige le cas confirmé où un trap T-45 aurait pu être lu
  `TouchAndGo` (crosse écrasée sur le pont lue comme "up" pendant l'événement en retard) sans le
  filet de sécurité du LQM DCS. Reste à revalider en mission live que ce nouveau repère géométrique
  ne fige jamais la lecture *avant* un contact réel sur un type dont l'offset de crosse serait encore
  imprécis (voir le point précédent) — un offset trop bas déclencherait `alt <= 0` prématurément,
  gelant la lecture quelques dixièmes de seconde trop tôt.
- **`NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75 m`** : vérifié par reconstruction manuelle contre 5
  rapports JSON du 5 septembre matin, et **confirmé en conditions live une seconde fois** le 5
  septembre soir sur 8 rapports supplémentaires (T-45/F-14B(U) humains cette fois) — plus aucun
  échantillon sous 3 m, plus aucune amplification artificielle (derniers échantillons à 3,0-5,6 m,
  angles -2,8° à +1,3° cohérents avec une reconstruction en mètres à écart quasi constant). Le doute
  restant est désormais plus étroit : le comportement est confirmé sur deux corpus indépendants
  (IA puis humain), mais 75 m n'a toujours aucune base doctrinale chiffrée (choisi par cohérence
  physique, ~1 s de vol à l'approche, pas calibré sur un corpus large), et le cas théorique d'un écart
  réellement dangereux et bref développé uniquement dans les tout derniers mètres reste à vérifier.
- Le collecteur `--positions-only` atteint-il réellement p99 <300 ms en usage prolongé et
  multi-recovery ?
- Chronométrer la procédure de déploiement/rollback (voir AGENTS.md, "Déploiement et rollback") sur
  une copie de staging avec le vrai wrapper de service et les permissions filesystème réelles :
  jamais exécutée en conditions réelles, l'objectif des cinq minutes n'est donc pas validé.
- Cadence de lecture LSO adaptative 100/200 ms hors zone de notation : à valider par A/B
  (`lso.exe cadence-ab`) avant toute promotion — voir "Hors-scope confirmé" ci-dessus, jamais
  décidée à ce jour.
- Validation live DCS-gRPC de la ligne de version serveur réellement déployée avant tout repin des
  stubs vers une révision Git immuable.
- Refaire les scénarios de validation Phase 6 (voir plus bas) sur une mission moins minimaliste :
  câbles 1-4, pattern puis finale, longue passe, simultané, respawn, reconnect/session, gaps, sans
  ACMI et V/STOL.
- Revalider en mission live le correctif de la remise de gaz en survol (`DECK_CROSSING_ALT_CAP_FT`) :
  confirmé le 5 septembre soir pour le cas déjà corrigé (survol haut ~460 ft, aucune remise de gaz
  haute classée bolter). Le test a montré que le plafond seul ne suffisait pas pour un survol bas
  (~28 ft) ou une crosse modélisée sous le pont ; voir l'entrée dédiée ci-dessous
  (`DECK_CONTACT_CONFIRMATION_ALT_M`/refus `GRADE:WO`) pour le correctif complémentaire, lui aussi
  non revalidé en mission live.
- Revalider sur données live les nouveaux seuils de notation (`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`,
  `OSCILLATION_MIN_SWING_DEG`/`OSCILLATION_MIN_REVERSALS`) : jamais rejoués sur un corpus de
  rapports live pour vérifier qu'ils ne masquent pas de vrais écarts ni ne déclenchent de faux
  positifs sur des approches réelles.
- **`GROOVE_ROLLOUT_MAX_BANK_DEG`/`GROOVE_ROLLOUT_MAX_TRACK_ANGLE_DEG` (= 15° chacun)** : testé
  unitairement, puis **confirmé une première fois en conditions live** le 5 septembre soir sur les 8
  passes humaines — entrée en groove toujours détectée entre 870 et 1 384 m, jamais en plein virage
  (bank au premier échantillon continu entre -7,4° et -14,5°, donc toujours sous le seuil de 15° une
  fois l'entrée retenue). Reste à confirmer que ces seuils n'excluent jamais à tort une entrée en
  groove sur un virage large ou un fort vent de travers non rencontré sur ce corpus. **Effet de bord
  découvert par la même occasion, désormais traité séparément** : la doctrine qui place l'entrée en
  groove après la porte 3/4 NM rendait cette porte structurellement invalide/hors-boîte sur un
  pattern Case I humain standard — voir plus bas l'entrée dédiée sur la relaxation de la porte 3/4
  NM, elle aussi non revalidée en mission live, avant de considérer ce raffinement comme
  complètement validé pour un usage humain.
- **Précision du proxy de route sol (`is_rolled_out`) : régression linéaire plutôt que deux points
  d'extrémité** (implémenté le 6 septembre 2026, suite à une recherche documentaire sur la façon
  dont les LSO US Navy détectent réellement le début du groove). La recherche a confirmé que le
  début du groove Case I est bien un événement chez NATOPS **et** chez deux historiens de
  l'aviation navale convergents (virage terminé, ailes à plat, aligné, en palier) — aucune remise
  en cause de l'approche événementielle déjà en place (`is_rolled_out`, bank + track angle). Deux
  pistes d'amélioration de la mesure elle-même ont été évaluées : (1) remplacer la comparaison des
  deux échantillons aux extrémités du buffer `gate_samples` par une régression linéaire sur toute
  la fenêtre (`track_velocity_regression`) — implémentée, aucun nouveau seuil `PROJECT-DERIVED`,
  amélioration pure de robustesse au bruit ; (2) ajouter un critère "on speed" (AoA) — écartée
  explicitement par l'utilisateur, l'AoA du projet n'étant qu'une approximation géométrique jamais
  exposée par DCS-gRPC, jugée insuffisamment fiable pour conditionner une détection. Reste à
  revalider en mission live que la régression ne décale jamais l'instant d'entrée en groove détecté
  de façon perceptible par rapport à l'ancienne méthode sur un corpus réel (aucun changement
  observé sur les corpus synthétiques des tests unitaires, mais jamais rejouée sur les rapports
  live du 5 septembre).
- `cargo audit` reste à exécuter dès qu'un outil autorisé est disponible localement (la CI l'exécute
  déjà).
- **Désynchronisation `wire_estimated`/`wire_estimation`** : corrigée et testée unitairement, puis
  **confirmée résolue en live** le 5 septembre soir — `wire_estimated == wire_estimation.wire` sur
  les 8 rapports du test humain, aucune récidive de désynchronisation observée. Reste ouvert : le
  biais de fond de l'estimation elle-même (voir le point P1 dédié "Estimation Rust du brin
  systématiquement décalée" ci-dessus), qui est un problème différent de la désynchronisation
  maintenant résolue.
- **Explosion `atan2` de `trajectory_deviations`** : voir `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M`
  ci-dessus, désormais confirmée résolue sur deux corpus live indépendants.
- Revalider sur données live le Cut sink-rate/bank-angle (`SINK_RATE_CUT_MPS = 8.0 m/s`,
  `BANK_ANGLE_CUT_DEG = 30°`) : contrairement à la plupart des autres seuils de ce module, celui-ci
  n'a **aucune base doctrinale chiffrée** à valider — seulement à confirmer, sur un corpus de
  rapports live variés (dont au moins une approche par gros temps/vent de travers, où le sink
  rate/bank réels sont naturellement plus élevés), qu'il ne déclenche jamais de faux positif sur un
  poser normal ni ne manque un vrai cas dangereux. Priorité avant toute promotion : le risque de
  faux positif est le plus élevé de tous les seuils du module, faute de nombre NATOPS à recaler
  dessus. **Premier contrôle fait le 5 septembre matin** contre 9 rapports d'un test live IA (6 posés
  + 3 remises de gaz) : aucun faux positif (sink rate max observé 6,75 m/s, 84 % du seuil ; bank max
  3,5°, 12 % du seuil), mais corpus entièrement IA avec des approches anormalement lisses. **Second
  contrôle le 5 septembre soir, cette fois humain**, apporte un premier vrai signal de calibration :
  le LQM DCS a jugé un sink rate de **6,0 m/s** (75 % du seuil `SINK_RATE_CUT_MPS`) comme fautif
  (commentaire `TMRDAR`, "too much rate of descent"), alors que notre Cut ne se serait déclenché qu'à
  8,0 m/s — premier point de calibration humain suggérant que le seuil actuel est possiblement trop
  permissif. Par ailleurs, une remise de gaz du même test a atteint 7,83 m/s (98 % du seuil) dans le
  1/4 NM sans qu'un Cut sur une remise de gaz ait de sens : à vérifier explicitement que
  `dangerous_sink_rate_or_bank` n'est évalué qu'après confirmation d'un toucher, jamais sur un
  survol/waveoff. Bank max observé sur ce test : 3,0 à 7,8° (12-26 % du seuil), toujours sans risque
  de faux positif apparent. Reste entièrement à faire : un enregistrement avec un pilotage plus
  agressif ou dégradé (vent fort, correction tardive) pour tester si le seuil déclenche au bon
  moment plutôt que seulement l'absence de faux positif.
- Revalider en mission live l'automatisation de `_OK_` (bande d'amplitude MOOSE-inspirée + fenêtre
  de temps de groove NATOPS 15-18 s) : deux points distincts à confirmer séparément. (1) La bande
  d'amplitude resserrée n'a, comme le Cut sink-rate/bank, aucune base NATOPS chiffrée — à vérifier
  qu'elle ne déclenche jamais `_OK_` à tort sur un poser simplement propre mais pas réellement
  parfait ; aucune des 8 passes humaines du 5 septembre soir n'en approche (aucun `OK`/`(OK)` du
  tout sur ce test avec la règle de gates alors en vigueur — voir plus bas l'entrée sur la relaxation
  de la porte 3/4 NM, qui change potentiellement ce constat sans avoir été rejouée sur ce corpus),
  donc toujours aucune preuve empirique de faux positif ni de vrai positif à ce stade. (2) La fenêtre de temps 15-18 s, elle, est bien `OFFICIAL`, mais
  appliquée aujourd'hui identiquement au T-45 malgré sa pente différente (3,0° vs 3,5°) — **premier
  signal concret le 5 septembre soir** : sur 5 valeurs de `groove_time_secs` mesurées (19,8 à 26,1
  s, T-45 et F-14 mélangés), aucune ne tombe dans la fenêtre 15-18 s, y compris le F-14 le plus
  rapide du test (19,8 s) — voir le point P1 dédié plus haut. Échantillon encore trop petit et
  entièrement séquentiel (aucune pression opérationnelle de recovery multiple) pour conclure que la
  fenêtre est mal calibrée plutôt que ce groupe de pilotes ayant simplement volé un pattern large,
  mais le signal justifie de trancher ce point avant de compter sur `_OK_` comme atteignable en
  usage humain normal.
- Confirmer en mission live que les événements DCS pertinents (contact/`Land`, LQM) se comportent
  pour l'AV-8B/Tarawa comme pour les porte-avions CATOBAR : seule la géométrie/les seuils V/STOL
  (voir [VSTOL.md](VSTOL.md)) ont été travaillés en détail, la sémantique événementielle côté Tarawa
  n'a jamais été spécifiquement vérifiée en direct.
- Politique de skew pendant que le porte-avions vire ou accélère : le contrat de télémétrie
  (100/300 ms) n'a jamais été spécifiquement exercé avec un porte-avions en virage ou en
  accélération plutôt qu'en ligne droite/vitesse constante.
- Comportement V/STOL réel non simulé : palier vertical (VL) et roulé (RVL) réels, rebond/double
  contact — seule la classification (contact suivi d'un départ → neutre, jamais `Bolter`) a été
  pensée en amont ; aucune de ces situations n'a été observée sur un vrai vol Tarawa.
- Géométrie exacte d'occupation du spot 7.5 (AV-8B/Tarawa) jamais vérifiée en conditions réelles,
  seulement calculée depuis les références géométriques de [VSTOL.md](VSTOL.md).
- Aucun manifeste `--baseline-manifest` n'a jamais été réellement rempli/authentifié contre un
  déploiement live et propagé jusqu'au rapport final : le 5 septembre soir confirme qu'un manifeste
  valide est bien accepté au démarrage (voir bug B5 en P1 ci-dessus) mais ressort entièrement `null`
  dans le JSON — build DCS, SHA-256 du DLL et des fichiers Lua déployés, versions de module avion
  restent donc à ce jour "non authentifiés" pour toute session live passée, cette fois pour une
  raison de bug confirmée plutôt que d'absence de procédure.

## Pistes de notation encore ouvertes

- **NC vs statut neutre dédié** : `grading_availability` (`available` / `unavailable_technical` /
  `unavailable_event_outcome`) distingue déjà, en JSON, une `NC` d'origine télémétrique d'une `NC`
  d'origine événementielle ; introduire un nouveau statut *pass_grade* dédié à "neutre par
  construction" (au lieu de fusionner les deux sous l'étiquette `NC` visible pilote) serait une
  décision de nommage/UX (quel libellé, quel impact sur le greenie board existant) plutôt qu'un
  correctif de clarté à faible risque — hors scope d'une implémentation automatique sans validation
  produit. Laissé ouvert pour une session dédiée si le besoin se confirme. Le bug `Bolter` sans
  preuve de toucher (corrigé, voir "Décisions encore ouvertes" ci-dessous) et le désaccord DCS/LSO
  qu'il produisait (`GRADE:WO` contre `Bolter`) restent un argument de plus en faveur d'une
  réflexion prochaine sur la granularité des statuts affichés au pilote, au-delà du seul `NC`.
- **AoA réellement lue depuis le cockpit** (`aircraft_draw_argument`) : abandonné, pas un refus
  définitif. Le draw argument correspondant a été confirmé **absent** du modèle 3D pour au moins le
  T-45C (vérifié via ModelViewer 2.0 contre l'installation client) ; aucune source primaire
  vérifiable n'a été trouvée pour les trois autres avions non plus. Ne pas relancer cette piste sans
  accès DCS live pour un balayage empirique de `UnitService.GetDrawArgumentValue`, ou une nouvelle
  vérification ModelViewer pour les types restants. Remplacé pour l'instant par la correction du
  vent sur l'approximation géométrique existante (voir [CHANGES.md](CHANGES.md)) — cette
  approximation elle-même montre un comportement aberrant à hauteur de pont sur le test du 5
  septembre soir (voir "Hypothèses non confirmées" ci-dessus), toujours en contexte affiché
  uniquement, jamais noté.

Explicitement écarté (pas une piste à reprendre sans nouvelle donnée) :

- Reconstruire une "fenêtre de remise de gaz" dynamique dépendante de la puissance moteur réelle :
  DCS n'expose pas les données nécessaires.
- Entraîner un modèle statistique sur d'anciens vols : aucun corpus de grades LSO humains alignés
  sur les traces DCS du projet n'existe ou n'est raisonnablement constituable.

## Scénarios de validation Phase 6 (checklist de non-régression avant toute promotion majeure)

1. CATOBAR nominal, câbles DCS 1 à 4 ;
2. passage pattern puis finale, sans verrouillage du câble ancien ;
3. AV-8B/Tarawa : palier vertical nominal, VL roulé, rebond/double contact, touch-and-go et
   go-around ;
4. `Land`/`RunwayTouch`/LQM absents, dupliqués, tardifs et réordonnés ;
5. hook argument à travers groove/finale pour chaque module CATOBAR supporté (F/A-18C, T-45, F-14) —
   T-45 et F-14B(U) désormais couverts par un test humain (5 septembre soir, polarité confirmée dans
   les deux sens ; la contamination de fenêtre par le contact pont qu'il a révélée est corrigée, voir
   "Décisions encore ouvertes" ci-dessous) ; F-14A/B et F/A-18C restent à couvrir par un
   enregistrement humain équivalent ;
6. brins 1-4 avec LQM DCS et vérité d'un observateur indépendant ;
7. porte-avions en ligne droite, en virage et en accélération, autour de 100/300 ms de skew ;
8. reconnect, rotation de mission, départ/respawn/changement de slot joueur, et homonymes ;
9. simultanéité Hornet/CVN et AV-8B/Tarawa ;
10. charge 40 joueurs/deux navires et stress à trois porte-avions ;
11. même recovery Hornet/CVN en mode hook indépendant vs `--legacy-inline-hook-sampling` ;
12. les deux modes hook avec `--no-acmi`, un RPC hook retardé et des timestamps source gelés ;
13. `--position-source unary` vs `buffered` par défaut sur la mission identique ;
14. délais de livraison bufferisée de 300 ms et 1 s, en vérifiant que les séquences intermédiaires
    capturées sont bien renvoyées et traitées ;
15. gel du producteur, dépassement de capacité, expiration de rétention, changement d'epoch et
    retry d'un même `after_sequence` — vérifier qu'aucune position n'est fabriquée ni perdue
    silencieusement ;
16. hook timeout/error/stale et passe longue >128 s ;
17. charge avec détecteurs actifs puis suspendus pendant le groove.

Pour chaque événement testé, corréler la charge utile brute et l'ordre d'arrivée à travers DCS,
DCS-gRPC, LSO et l'ACMI, sans normaliser le log source avant de l'avoir préservé tel quel.
