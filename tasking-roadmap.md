# Tasking & roadmap — DCS-gRPC-lso

> Idées, choix techniques ouverts, arbitrages à faire et bugs connus non résolus ou non revalidés.
> Ne contient que des points **encore en suspens** — dès qu'un point est résolu et revalidé (ou
> tranché sans besoin de revalidation), il est retiré d'ici ; l'essentiel synthétique de ce qui a
> été fait migre vers [CHANGES.md](CHANGES.md) (et [AGENTS.md](AGENTS.md) si ça touche l'état
> durable du système) — voir [AGENTS.md](AGENTS.md), "Règles de maintenance des documents markdown
> racine", pour la règle complète. Pour le détail narratif des sessions de test passées (dates,
> corpus, discussions de conception ayant mené aux points encore ouverts ci-dessous), `git log`/
> `git show` sur les commits correspondants fait foi ; ce document ne le duplique pas. Dernière
> purge/fusion : 7 septembre 2026.

## À faire en priorité (P0)

- **Réduire au maximum les `Grading unavailable` sans transformer une absence de preuve en note
  inventée.** Le défaut de produit n'est pas seulement la fréquence des indisponibilités : des
  informations certaines (issue, qualité de la portion observée, grade DCS) sont aujourd'hui
  aplaties dans un unique `PassGrade::Incomplete`, puis présentées au pilote comme si rien
  d'exploitable n'avait été mesuré. Les deux corpus humains du 7 septembre totalisent 3 rapports
  indisponibles sur 16 : 2 avec `unconfirmed_arrest`, dont le faux cas WO/WO/trap que la correction
  de segmentation courante doit éliminer, et 1 avec `invalid_telemetry + unconfirmed_arrest` ; ils
  n'exercent toutefois pas tous les chemins théoriques. Le chantier doit séparer durablement
  **issue**, **évaluation d'approche**, **périmètre observé** et **confiance technique**, puis traiter
  chaque cause selon sa matérialité réelle :

  **Tranche locale implémentée, revalidation live encore requise** : l'âge de livraison bufferisé
  ne rend plus une capture continue invalide ; le watchdog bufferisé attend dans la limite de la
  rétention annoncée ; l'ancien pattern graphique est compacté sans `BufferLimit` de notation ; et
  JSON/SQLite/Discord/board distinguent désormais périmètre, couverture, points et source de
  fallback. Une approche partielle conserve son appréciation sans points et l'issue reste séparée.
  Restent ouverts dans ce chantier : drain vers queue locale dédié, couverture matérielle des
  observations invalides isolées et des gates manquantes, journal d'événements persistant avec
  grâce tardive, suppression prouvée des faux départs, et éventuelle promotion cinématique après
  vérité terrain gel/rebond. Les huit points ci-dessous restent donc la spécification active.

  1. `TelemetryGap` : pour la source bufferisée, un `delivery_age` élevé avec capture continue et
     séquences lecteur contiguës doit dégrader la santé, jamais suffire seul à retirer la note. Une
     vraie rupture de capture ou perte de séquence dans le segment noté reste bloquante. Le
     watchdog bufferisé doit tenter de récupérer les snapshots pendant une fenêtre compatible avec
     la rétention du ring au lieu de terminer automatiquement après 2 s ; si le curseur reprend
     sans perte, la passe reste exploitable. Isoler idéalement la lecture bufferisée dans une tâche
     qui draine continuellement la source vers une queue locale, afin que vent, ACMI, rendu ou
     autres RPC ne retardent pas l'acquisition. Ne pas modifier la politique unary sans preuve.
  2. `InvalidTelemetry` : remplacer « une frame invalide dans le groove annule toute la passe » par
     une décision de couverture en fin de track. Un trou court entièrement encadré par de vrais
     samples valides dans <=300 ms reste diagnostique ; aucune position ne doit être fabriquée. Une
     gate ne peut être utilisée que depuis un bracket réel conforme. Pour une observation source
     sans temps, utiliser si possible `capture_tick`/séquence et les voisins horodatés afin de
     borner son segment, sans substituer le temps de réception. Les séries longues ou impossibles à
     borner dans le segment noté restent indisponibles. Un verdict déjà irréversible (`Cut`, par
     exemple) ne doit pas être remplacé par NC si les données manquantes ne peuvent plus changer ce
     verdict.
  3. `InsufficientGates` : détecter plus tôt les tentatives et accepter comme preuve de couverture
     la trajectoire continue valide autour des distances requises, plutôt que dépendre uniquement
     de trois objets gate ponctuels. Le seuil de bracket 300 ms reste inchangé. Si l'enregistrement
     commence réellement trop tard ou qu'une zone nécessaire n'est pas observée, exposer une
     évaluation partielle (`observed_from_distance_m`, couverture manquante), sans points, plutôt
     qu'un NC opaque. Corriger aussi le message « fewer than three gates », faux lorsque 3/4 NM est
     légitimement exclue après roll-out.
  4. `UnconfirmedArrest` : rendre le flux d'événements persistant/reconnectable au niveau session,
     conserver un court journal par IDs/temps DCS et laisser une grâce de corrélation avant
     finalisation, afin qu'un LQM tardif ne soit pas perdu entre deux tracks. La preuve cinématique
     structurée peut lever l'indisponibilité avec confiance `medium`, sans brin certain ni bonus de
     câble, seulement après des tests discriminant trap, bolter, T&G, remise de gaz, rebond,
     disparition et unité gelée ; le second corpus valide un trap et trois non-traps mais ne couvre
     pas encore tous ces risques. Un grade DCS sans `WIRE#` peut être montré comme fallback DCS,
     jamais converti seul en grade/points projet.
  5. `BufferLimit` : réserver la capacité aux gates/groove/touchdown, compacter ou faire tourner
     uniquement l'ancien pattern non noté, et ne jamais retirer une note pour la seule troncature
     d'un graphique. Attribuer les pertes réelles du ring source au segment par leurs séquences et
     bornes temporelles ; seules celles pouvant toucher le segment noté sont bloquantes.
  6. `EventStreamUnavailable` et `Grading::Unknown` : une panne événementielle ne doit pas effacer
     une approche positionnelle complète. Publier une `approach_grade` indépendante avec issue
     inconnue et points retenus ; supprimer des surfaces pilote les faux départs n'ayant ni groove,
     ni gate significative, ni événement d'issue. Une vraie finale sans issue doit devenir
     `approach_only`, pas être confondue avec « aucune approche reconnaissable ».
  7. UX/contrat : conserver `grading_availability`, mais ajouter de façon additive un périmètre
     (`full`/`partial`/`outcome_only`/`none`), la couverture observée/manquante, l'éligibilité aux
     points et la source de fallback (`project`/`dcs_lqm`/`geometry`/`none`). Discord, PNG, SQLite et
     board doivent afficher en priorité : note projet complète ; sinon note projet partielle sans
     points ; sinon grade DCS explicitement étiqueté ; sinon issue seule. Les issues certaines WO,
     B, T&G et trap/brin restent visibles même si l'approche est techniquement partielle. Réserver
     la formule nue `Grading unavailable` au seul cas où aucune information utile ne subsiste.

  Implémenter par étapes revues et testées : d'abord la sémantique source bufferisée et la
  préservation des issues certaines, puis la matérialité/couverture, les surfaces pilote, la
  résilience événementielle et enfin l'éventuelle promotion cinématique. Chaque fallback doit être
  sans points tant que les données requises ne sont pas complètes ; ne pas relever 300 ms,
  interpoler une longue coupure, assimiler les évictions du ring à une perte lecteur, ou confondre
  grade DCS et `project-derived-v4`. Revalidation live obligatoire avant de considérer ce P0 clos.

- **Rendre le PNG de pattern lisible lorsqu'une même track contient plusieurs circuits sans issue
  terminale observable.** Le corpus humain `.ignore/tests-20260907-3-human` confirme un cas précis,
  `LSO-20260907-220813-Justice-s1788809129-g8-p1001014-c5160-t2891150-pattern.png` : la track est
  armée à `2570,22` / 3,375 NM, dure ensuite 320,85 s (`2570,30` à `2891,15`) et dessine deux
  circuits complets plus une longue branche avant la finale réellement évaluée. Les deux premiers
  survols passent devant le point de toucher vers `2632,10` (`x = -3216,5 m`, altitude relative
  134,8 m) et `2778,75` (`x = -2315,8 m`, altitude relative 118,1 m) : aucun n'entre en groove,
  n'approche le pont, ne produit de contact ni de LQM `GRADE:WO`. Le code applique donc à juste
  titre son reset de minimum « pattern: plane moving away » et conserve la même track. La vraie
  finale n'entre en groove qu'à `2871,45`, à 859,7 m, puis se termine en T&G crosse haute. Les
  trois gates de cette dernière finale sont valides et le verdict `NoGrade`/2,0 points est
  exploitable ; le défaut démontré concerne la restitution graphique du contexte antérieur, pas
  la télémétrie, la note, ni la correction de segmentation des `GRADE:WO` (celle-ci est par ailleurs
  exercée favorablement dans le même corpus par deux WO distincts suivis d'un trap câble 2 distinct,
  sans généraliser cette preuve F-14B(U) à tous les modules/scénarios).

  **Solution recommandée, limitée au rendu :** conserver intégralement les données et la logique
  métier de `Track`, mais segmenter `pattern_datums` en branches continues à partir des inversions
  approche/départ déjà observables, puis choisir comme branche principale la plus récente qui
  contient l'entrée en groove ou, à défaut, l'issue/contact retenu. Afficher cette branche en
  couleurs normales ; afficher les branches antérieures en gris fin/atténué, sans jamais relier la
  fin d'une branche au début de la suivante, avec une légende telle que « 2 circuits antérieurs ».
  Si aucune branche ne contient groove, gate significative ou issue, ne pas publier de PNG pilote
  et conserver seulement un diagnostic de faux départ. Ne pas fermer automatiquement la track sur
  les deux survols hauts de ce corpus : sans groove, contact ou LQM, ils sont indiscernables d'un
  pattern/overhead légitime et une fermeture métier risquerait de perdre la finale suivante. Ne pas
  supprimer les points historiques du JSON/ACMI, ne pas modifier le grading, ne pas inventer une
  séparation temporelle depuis le temps de réception, et ne pas utiliser une distance fixe seule
  comme preuve de waveoff.

  **Contexte d'implémentation :** le PNG est rendu depuis `TrackResult::pattern_datums` dans
  `src/draw.rs`, tandis que le JSON schema-v3 ne sérialise actuellement que `datums` (sous-échantillonnés
  hors segment noté), pas `pattern_datums`. Extraire une fonction pure de segmentation/sélection,
  testable sans backend graphique, et produire un petit diagnostic additif (`pattern_branch_count`,
  `primary_pattern_branch`, motif de sélection, branches masquées/atténuées). Réutiliser autant que
  possible les notions existantes d'inbound, `groove_entry_time`, contact/issue et croissance de
  distance >150 m, mais sans leur attribuer une nouvelle sémantique métier. Vérifier aussi que la
  compaction récente de l'ancien pattern conserve les marqueurs nécessaires à cette segmentation.

  **Critères d'acceptation déterministes :** (1) pattern simple nominal inchangé ; (2) deux circuits
  hauts puis T&G reproduisant les temps/positions ci-dessus : dernière branche colorée, deux branches
  grises, aucune ligne de raccord artificielle ; (3) `GRADE:WO` suivi d'un nouveau circuit : deux
  rapports/tracks restent la source de vérité, aucune fusion par le renderer ; (4) bolter puis
  nouveau circuit ; (5) trap avec rollout ; (6) faux départ sans groove/gate/issue non publié sur
  les surfaces pilote ; (7) pattern tronqué/compacté sans panic ni promotion en `BufferLimit` ;
  (8) rendu CATOBAR et V/STOL sans régression de cadrage. Rejouer visuellement le PNG signalé et les
  autres PNG `g8`, puis revalider en mission live un overhead simple, un waveoff sans LQM et un
  `GRADE:WO` explicite. Un replay local valide le rendu seulement, jamais le comportement DCS live.

- **Dégradation sévère et systémique du cadencement position pendant le test humain du 6 septembre
  après-midi (`Justice-20260906`, 4 passes, code/commit actuel, `--position-source buffered`) —
  a coûté sa note à un trap réel confirmé par DCS.** Sur les 4 rapports (mêmes session/génération/
  porte-avions, ~11 minutes), `position_poll_p95_latency_ms` vaut 857-880 ms et
  `position_poll_p99_latency_ms` 911-922 ms **sur les 4 passes sans exception**, `gap_p50_ms` autour
  de 180-200 ms (cible 20 Hz = 50 ms), et `position_poll_max_latency_ms` frôle ou dépasse 1000 ms sur
  chacune (969, 934, 943, **1000** ms). C'est très au-dessus de ce qu'un test IA sur le même commit,
  la veille (6 septembre, mêmes seuils de contrat), avait mesuré (`max_sample_gap_ms` ~120-160 ms,
  `gap_p99_ms` ~120 ms) — la dégradation n'est donc pas un trait normal du build, mais propre à cette
  session précise (machine/charge différente ? DCS piloté en client sur la même machine que le
  serveur, contrairement au test IA ?). Conséquence concrète : sur 2 des 4 passes (la 1ʳᵉ et la
  **4ᵉ, un poser réellement accroché brin 3 confirmé par le LQM DCS**), un échantillon a dépassé le
  seuil dur de 1000 ms *à l'intérieur du segment noté*, déclenchant `TelemetryGap` /
  `grading_availability: unavailable_technical` — message pilote Discord "Grading unavailable:
  TelemetryGap. This is a measurement limitation, not a pilot failure." Comportement conforme au
  contrat de télémétrie (voir AGENTS.md, "Contrat de télémétrie" : jamais inventer un point sous un
  gap >1 000 ms), donc **pas un bug de règle de notation** — mais l'obtention réelle de ce gap coûte
  la note d'un vrai trap, ce qui en fait une régression de disponibilité prioritaire à instrumenter.
  Signal corroborant à qualifier : `recovery_telemetry.overflow_count` vaut 2962/2661 sur ces deux
  rapports (`snapshots_received - 600` exactement, `high_water_mark` à 600/600,
  `lost_snapshots: 0`). Ce compteur prouve du churn/une éviction interne du ring, **pas** des
  snapshots perdus par le lecteur : aucune rupture de séquence n'est démontrée. **Topologie
  confirmée par l'utilisateur
  (6 septembre 2026)** : `lso.exe` tournait sur le serveur DCS dédié lui-même (boucle locale
  `127.0.0.1:50051`, comme pour le test IA de la veille) ; le client de Justice était sur un poste
  distinct, connecté au serveur par le réseau. Le canal gRPC `lso.exe`↔serveur était donc en boucle
  locale dans les deux tests (IA et humain) — **le réseau client DCS↔serveur est hors de cause pour
  ce canal**, ce qui élimine une piste et recentre l'hypothèse sur une charge serveur dédié plus
  lourde avec un vrai client humain connecté (plus de trafic réseau/état à répliquer côté DCS
  qu'avec des IA seules) qu'avec le test IA de la veille, faisant concourir `lso.exe` pour du CPU sur
  la même machine. Le test humain `-vv` réalisé ensuite confirme une capture source intacte mais
  une livraison tardive (voir la dernière mise à jour ci-dessous). La mesure CPU/réseau du serveur
  dédié et un A/B avec/sans traces restent nécessaires pour attribuer cette latence à la contention
  DCS plutôt qu'au niveau de journalisation de `lso.exe`.

  **Mise à jour 7 septembre 2026** : un second corpus déposé par l'utilisateur (même pilote
  `Justice`, même soirée du 6 septembre, `.ignore/tests-20260906-human/`, hors dépôt Git) — mais une
  session distincte de celle ci-dessus (6 passes réparties en deux groupes de 3, ~21h00 et ~23h15,
  pas 4 passes sur ~11 minutes) — **reconfirme la dégradation sur un échantillon plus large sans
  jamais atteindre le seuil dur cette fois** : `position_poll_p95_latency_ms` 659-785 ms,
  `position_poll_p99_latency_ms` 747-909 ms, `position_poll_max_latency_ms` 768,6-972,8 ms et
  `max_scoring_sample_gap_ms` 829,9-949,9 ms sur les 6 passes sans exception — toujours très
  au-dessus de la baseline IA (~120-160 ms), mais aucune des 6 n'a cette fois franchi le seuil dur de
  1 000 ms (la plus proche, 972,8/949,9 ms, en reste à ~5 %). Aucun `TelemetryGap` déclenché sur ce
  corpus. Renforce la conclusion déjà écrite : la dégradation est un trait systémique de cette
  configuration (client humain + serveur dédié), pas un incident isolé, mais son passage au-dessus du
  seuil dur de 1 000 ms qui coûte réellement une note semble marginal/intermittent plutôt que garanti
  à chaque passe. Le corpus `-vv` décrit juste après a depuis séparé capture et livraison.

  **Mise à jour après instrumentation, second corpus humain du 7 septembre 2026**
  (`.ignore/tests-20260907-2-human`, `-vv`) : les quatre tracks publiées capturent toutes à
  20 Hz avec `max_capture_gap_ms = 50 ms`, zéro intervalle source manqué et zéro perte de séquence
  lecteur, malgré 2 210 à 7 224 évictions de capacité. La dégradation est donc bien un retard de
  livraison : p95 690-830 ms, maximum 870-1 020 ms. Les deux dépassements de 1 000 ms sont restés
  hors segment noté (`scoring_invalid_samples = 0`, maximum noté 870-990 ms), donc aucune note n'a
  été retirée par la télémétrie cette fois. La séparation capture/livraison/perte est ainsi validée
  live sur ce scénario et confirme que les évictions seules ne sont pas des pertes ; la cause de la
  latence reste ouverte faute de mesure CPU serveur et d'A/B sans les 14,7 Mo de traces `-vv`.

## P1 — bugs confirmés à corriger, décisions à prendre

- **Attribution temporelle des observations unité invalides : revalidation live requise.** Le
  correctif conserve maintenant temps source/séquence/entité/statut
  et n'invalide que `[entrée groove, touchdown]`; après-touch et avant-groove restent diagnostics,
  temps source absent reste indéterminé et conservateur. Rejouer un cas semblable à 09:56 avec le
  nouveau binaire et vérifier que les statuts source exacts, les 11 instants et leur arrivée tardive
  apparaissent sans `TimeWentBackwards` inventé. La santé capture/livraison/perte, elle, a été
  revalidée live sur le second corpus du 7 septembre (voir P0) ; aucune observation unité invalide
  n'y était présente, donc il ne teste pas cette attribution.
- **Estimation Rust du brin systématiquement décalée par rapport à l'événement DCS — les deux
  moitiés du correctif sont désormais implémentées (6 septembre 2026), reste entièrement à
  revalider en mission live.** Mécanisme confirmé le 5 septembre soir : sur un rapport où Rust
  (brin 4, confiance `medium`) divergeait de DCS (brin 1), le `runway_touch` DCS arrivait après 4
  franchissements géométriques successifs, et l'estimateur retenait alors systématiquement le
  **dernier brin franchi avant l'événement DCS** plutôt que celui réellement accroché. Deux
  correctifs indépendants sont maintenant en place : (1) `wire_estimate_at` (`src/track.rs`)
  n'accorde plus jamais `confidence: "high"` sans brin confirmé par DCS (`WIRE#` du LQM) — corrige
  le cas confirmé où un survol/bolter sans accrochage produisait une confiance "high" sémantiquement
  fausse. (2) `wire_estimate_at` préfère désormais le premier franchissement de brin survenu au
  moment ou après le début d'une décélération horizontale soutenue détectée en continu
  (`observe_horizontal_deceleration`, `WIRE_ARREST_DECELERATION_MPS2 = 5,0 m/s²` sur
  `WIRE_ARREST_DECELERATION_MIN_CONSECUTIVE_SAMPLES = 2` échantillons, `PROJECT-DERIVED`, non
  chiffré NATOPS) plutôt que le dernier franchissement avant l'événement DCS — distingue "encore en
  l'air, survole les seuils de brin" de "déjà accroché, entraîné au-delà par l'élongation du câble".
  Une première tentative (corréler sur `first_hook_ground_contact_time`, un seul instant géométrique
  plutôt qu'un proxy de décélération) avait été abandonnée car elle cassait le fixture
  `wire_4_01_FA18C` ; la nouvelle approche par décélération continue résout ce même fixture
  correctement (`cargo test tests::wire_4_01`) et sans régression sur les 4 autres fixtures de
  brin, plus deux nouveaux tests dédiés
  (`wire_estimate_prefers_the_crossing_at_deceleration_onset_over_a_later_stretch_crossing`,
  `wire_estimate_ignores_a_crossing_recorded_while_still_airborne_before_any_deceleration`).
  **Limite de validation locale importante** : ce proxy dépend de `plane.velocity`, que seul le
  chemin gRPC live remplit (`Transform::from` du message `Velocity`) — `lso.exe file` (rejeu ACMI/
  Tacview, dont tous les fixtures `tests/recordings/*.zip.acmi`) laisse `velocity` à zéro (même
  lacune préexistante que `touchdown_horizontal_speed_mps`, dont le test attend déjà `0.0` sur
  rejeu), donc **aucun enregistrement local ne peut exercer ce nouveau proxy** : il retombe sur
  l'ancien comportement (dernier franchissement) sur toute source ACMI, sans régression, mais aussi
  sans validation contre un vrai trap enregistré. `lso.exe cadence-ab` n'est pas non plus applicable
  à ce point : il ne rejoue que la géométrie gates/trajectoire depuis `datums`, jamais les
  franchissements de brin ni la vitesse. **Seule une nouvelle session live peut valider ce
  correctif** : vérifier sur un prochain trap enregistré (1) que `arrest_deceleration_onset_time`
  (nouveau champ diagnostic JSON, `wire_estimation`) est bien détecté proche du contact réel, (2)
  que le brin rapporté converge alors avec celui du LQM DCS plus souvent qu'avant sur les cas où ils
  divergeaient jusqu'ici.

  **Mise à jour 7 septembre 2026 — première preuve live du correctif, et deux constats
  supplémentaires plus sérieux que prévu.** L'utilisateur a déposé les 6 rapports JSON/PNG plus le
  log `-vv` complet d'une session humaine du 6 septembre au soir (pilote `Justice`, F-14B(U),
  `.ignore/tests-20260906-human/`, hors dépôt Git) : 4 T&G/bolters (`hook_up_near_deck`) et 2
  arrestations confirmées par le LQM DCS (`WIRE# 1` les deux fois). Le log couvrait en réalité
  plusieurs mois (14 juin-6 septembre) dans un seul fichier `lso.log` de 106 Mo, **encodé pour moitié
  en UTF-8 et pour moitié en UTF-16LE** (bascule visible autour du 5 septembre, ~13h) — probablement
  `Tee-Object -FilePath ... -Append` dans `run-live-buffered.ps1` qui prend l'encodage par défaut
  (UTF-16LE en PowerShell 5.1) sur un fichier commencé ailleurs en UTF-8 ; un `grep`/`Select-String`
  naïf ne voit donc qu'une partie du log. **Corrigé le 7 septembre 2026** : `Tee-Object` dans
  `.ignore/run-live-buffered.ps1` force désormais `-Encoding utf8`. Comme chaque exécution du script
  écrit dans son propre fichier horodaté (`$runStamp-$PositionSource.log`), ce `lso.log` unique
  couvrant plusieurs mois n'a pu être obtenu que par une concaténation manuelle de plusieurs fichiers
  de sessions séparées ; si une telle concaténation est refaite pour un futur dépôt de preuve, la
  faire avec un outil qui préserve l'UTF-8 (ex. `Get-Content -Raw | Out-File -Encoding utf8`), pas un
  simple `cat`/`type` qui recopierait des encodages mixtes.

  (1) **`arrest_deceleration_onset_time` est bien détecté sur les deux arrestations confirmées**
  (3694.87 s et 11757.19 s), à partir d'une vraie décélération continue (`plane.velocity` bien
  peuplé en live, confirmant que le proxy est actif hors ACMI comme prévu) — mais **la porte de
  corrélation temporelle en amont bloque les six passes sans exception, avant même d'atteindre la
  sélection par décélération** : `wire_estimate_at` retourne `wire: None` /
  `wire_crossing_not_time_correlated_with_event` sur les 6/6 rapports (`event_lag_ms` observés :
  505,5 / 554,5 / 770,9 / 804,3 / 864,8 / 1953,3 ms), très au-dessus des `SAMPLE_GAP_WARNING_MS`
  (300 ms) réutilisés pour cette vérification. **Sur ce corpus, l'estimation Rust du brin est donc
  actuellement non-fonctionnelle à 100 %** (0/6, y compris sur les deux arrestations dont le LQM
  confirme pourtant `WIRE# 1`) — plus sévère que le biais de -1 brin déjà connu, qui produisait au
  moins un numéro (faux). Cette porte n'a pas été touchée par le correctif du 6 septembre (elle
  s'applique avant la sélection par décélération) ; elle réutilise `SAMPLE_GAP_WARNING_MS`, pensé à
  l'origine pour la fraîcheur générale de télémétrie, pas pour la corrélation événement DCS/brin.

  (2) **Le log `-vv` révèle le mécanisme complet, sur les deux arrestations confirmées** (mêmes
  pilote/avion, même soirée — deux échantillons seulement, donc à reconfirmer sur un corpus plus
  divers) : la vitesse horizontale reste quasi constante (variation < 1 m/s² de bruit) pendant que le
  crochet franchit géométriquement les 4 brins en ~0,62-0,66 s, et la décélération réellement
  significative (>5 m/s² soutenue) ne démarre que **~0,9-1,0 s après le franchissement du brin 1**
  (écarts mesurés : 0,990 s et 0,895 s) — donc après que les 4 brins ont déjà été franchis
  géométriquement dans les deux cas. Le brin réellement accroché (brin 1, confirmé LQM) est le
  *premier* franchi, pas le dernier ; le crochet continue de glisser sur les brins suivants pendant
  cette phase de "roulement libre" avant que la tension du câble ne produise une décélération
  mesurable. **Conséquence pratique** : `WIRE_ARREST_ONSET_TOLERANCE_S = 0,5` (choisi sans base
  empirique) est environ deux fois trop court pour rattacher l'onset au bon brin sur ce corpus — avec
  une tolérance couvrant ~1,0-1,2 s, les 4 franchissements entreraient dans la fenêtre et le premier
  (brin 1) serait sélectionné, correctement, dans les deux cas.

  **Correctif implémenté le 7 septembre 2026** (validé avec l'utilisateur) : (a)
  `WIRE_ARREST_ONSET_TOLERANCE_S` élargi de 0,5 à 1,2 s (toujours `PROJECT-DERIVED`, calibré sur
  seulement 2 échantillons du même pilote/avion — à reconfirmer sur un corpus plus large) ; (b) la
  porte de corrélation événement/brin dans `wire_estimate_at` (`src/track.rs`) s'appuie désormais sur
  `arrest_deceleration_onset_time` quand il est disponible plutôt que sur le dernier franchissement
  brut — repli inchangé sur l'ancienne vérification (dernier franchissement vs événement,
  `SAMPLE_GAP_WARNING_MS`) quand aucun onset n'est détecté (bolter/T&G/gap). Testé unitairement avec
  un nouveau test dédié reproduisant la forme exacte du corpus live
  (`wire_estimate_correlates_via_deceleration_onset_when_the_last_crossing_lags_the_event_too_far`),
  sans régression sur les 5 fixtures ACMI ni les tests existants (`cargo test` 228, `cargo fmt`/
  `cargo clippy -D warnings` propres). Le second corpus humain du 7 septembre exerce bien cette
  variante : l'onset du trap final est détecté 350 ms avant l'événement, mais deux `GRADE:WO` et le
  trap ont été fusionnés par un bug de segmentation désormais corrigé. Les seuls franchissements
  conservés venaient alors du premier waveoff, et le `WIRE# 2` final avait été rejeté comme LQM
  dupliqué ; ce corpus confirme donc l'onset live mais ne valide toujours pas l'estimation de brin
  bout en bout ni l'absence de faux positif de la tolérance à 1,2 s. Refaire un trap isolé après le
  correctif de segmentation.

  **Historique de la divergence Rust/DCS avant ce correctif (test IA du 6 septembre après-midi, 6
  arrests)** : divergence observée sur 3 estimations sur 5 nommées (F14-2-1 DCS 4/Rust 3 confiance
  `medium` ; F/A-18C-2-1 DCS 4/Rust 3 confiance `high` ; F/A-18C-1-1 DCS 1/Rust 2 confiance `high`) ;
  concordantes sur les 2 autres (F14-3-1 3/3, F14-4-1 1/1, toutes deux `high`) ; une sixième
  estimation restait `insufficient`. Deux enseignements retenus pour le correctif ci-dessus : (1) le
  biais n'est pas systématiquement "Rust trop haut" — ce corpus montrait les deux sens (2× Rust en
  dessous de DCS, 1× Rust au-dessus). (2) Un mode de confiance "high" trompeuse distinct de celui
  déjà corrigé le 6 septembre matin existait quand DCS confirmait bel et bien un brin (arrêt réel)
  mais un brin *différent* de celui nommé par Rust — 2 des 3 divergences ci-dessus affichaient quand
  même `confidence: "high"` sur le brin erroné. **Ce point est devenu sans objet** : les surfaces
  pilote (Discord, PNG, dashboard) n'affichent jamais la confiance à côté du numéro de brin
  (`pilot_facing_outcome`, voir AGENTS.md) — elle n'existe que dans le JSON complet et l'API SQLite
  privée (loopback), réservées au diagnostic. Pas de plafonnement `"high"`→`"medium"` supplémentaire
  à ajouter pour cette seule raison ; le correctif par décélération ci-dessus s'attaque directement à
  la cause du mauvais numéro plutôt qu'à l'étiquette de confiance qui l'accompagnait.
- **`baseline_manifest` sérialisé entièrement `null` dans le JSON malgré un manifeste valide fourni
  au démarrage.** Le manifeste (6 clés, aucune erreur de validation) est accepté puis perdu entre
  `run.rs` et `RecoveryReport`, ou le champ du rapport est alimenté depuis une source jamais remplie.
  La provenance de build (`lso_commit`/`lso_dirty`/`dcs_grpc_version`), elle, est correcte. À noter
  aussi : un manifeste chargé une fois au démarrage ne peut pas suivre un changement de mission en
  cours de session (observé le 5 septembre : `_A` → `_B` après coup) — envisager d'horodater la
  lecture du manifeste dans le JSON, ou de le recharger à chaque nouvelle session DCS. Reconfirmé
  `null` sur les 6 rapports du corpus humain `Justice` du 7 septembre 2026 (aucune information
  nouvelle, juste une récidive de plus).
- **Référence de vent incohérente : `180°/0,0 m/s` intermittent — cause désormais fortement
  circonscrite (6 septembre 2026, après-midi, test IA 8 recoveries, vent mission 5 nds/360° vrai).**
  Grâce à l'instrumentation `wind_reference_probes` ajoutée le 6 septembre matin, les trois requêtes
  `GetWind` de chaque rapport sont maintenant comparables individuellement sur ce corpus : la probe
  **haute altitude** (~103-113 m, altitude avion) renvoie la valeur cohérente (`0°/4,26-4,34 m/s`) sur
  **8 rapports sur 8, sans exception** ; la probe **basse altitude** (~0 m, niveau du pont — utilisée
  pour l'interpolation AoA) est aberrante (`180°/0,0`) sur **5 rapports sur 8** ; la requête
  **séparée de fin de rapport** (position du navire, également quasi niveau mer) est aberrante sur
  **2 rapports sur 8** — et ces deux sous-ensembles ne se recoupent pas systématiquement (un rapport
  a une probe basse bonne mais une requête de fin de rapport aberrante, et inversement). La
  corrélation nette est donc **avec l'altitude de la requête, pas avec laquelle des trois elle est** :
  toute requête `GetWind` à une altitude quasi nulle (mer/pont) est intermittemment peu fiable (7
  occurrences aberrantes sur 16 requêtes near-zero-altitude ce test), tandis qu'une requête à
  l'altitude de vol reste fiable à 100 %. `wind_reference_established` reste `true` dans tous les cas
  aberrants observés : le RPC répond `Ok` avec `180°/0,0` sans jamais faire échouer l'appel, donc
  bien une incohérence côté DCS/DCS-gRPC quant au vent près du pont, pas un échec réseau ni un bug de
  notre propagation `Result`. **Reste à faire** : (1) confirmer cette corrélation altitude sur un
  corpus supplémentaire (celui-ci n'a que 3 altitudes distinctes réellement testées : ~0 m navire/
  pont deux façons différentes, et ~110 m avion — jamais une altitude intermédiaire) ; (2) décider
  avec l'utilisateur si un garde-fou de plausibilité (ex. rejeter `heading == 180 && speed == 0,0`,
  ou ne pas interroger à une altitude sous un plancher et réutiliser la probe haute) doit être ajouté
  — décision de conception, pas un correctif à appliquer unilatéralement ; (3) si possible, isoler
  côté DCS-gRPC/mission (hors périmètre de ce dépôt) pourquoi `GetWind` dégrade près du niveau de la
  mer.

  **Mise à jour 7 septembre 2026 (corpus humain `Justice`, 6 recoveries, 6 septembre soir)** :
  nouvelle occurrence isolée, avec un profil différent du test IA de l'après-midi précédent. Sur les
  6 rapports, les deux probes `wind_reference_probes` (haute et basse altitude) sont **toutes les 12
  cohérentes** (`32°/1,2-8,2 m/s`, jamais `180°/0,0`) ; seule la requête **séparée de fin de rapport**
  est aberrante, et seulement sur **1 rapport sur 6** (`wind_heading_deg`/`wind_speed_mps` =
  `180°/0,0` sur `LSO-20260906-231534-...json`, contre `32°/~1,5-1,9 m/s` sur les 5 autres). Sur ce
  corpus précis, c'est donc l'inverse du test IA (où la probe basse altitude était la plus souvent
  fautive) : une nouvelle confirmation que **n'importe laquelle** des trois requêtes near-zero-altitude
  peut être touchée de façon apparemment aléatoire, cohérent avec l'hypothèse déjà retenue
  (corrélation avec l'altitude de la requête, pas avec son identité), mais renforce aussi le besoin
  d'un garde-fou de plausibilité générique (point (2) ci-dessus) plutôt qu'un correctif ciblé sur une
  seule des trois requêtes.

  **Second corpus humain du 7 septembre 2026 (`tests-20260907-2-human`)** : les trois références
  établies ont des probes haute et basse cohérentes (`91°`, respectivement `5,49-6,10 m/s` et
  `1,14-1,42 m/s`). La requête séparée de fin de rapport renvoie pourtant `180°/0,0 m/s` sur le
  deuxième rapport alors que ses deux probes sont valides ; la quatrième track n'a pas établi de
  référence faute d'entrée en groove. Cette nouvelle occurrence renforce le caractère intermittent
  de l'anomalie near-zero/report-time, sans fournir encore une règle de remplacement sûre.
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
  trois explications est la bonne. **Mise à jour 6 septembre après-midi, corpus 100 % IA** : les 6
  passes accrochées du test IA mesurent `groove_time_secs` entre 19,5 et 22,2 s (F-14 : 20,3-21,1 s ;
  F/A-18C : 19,5-22,2 s) — à nouveau jamais dans `15,0..=18,0`, sur une plage quasiment identique au
  corpus humain (19,8-26,1 s). Un pilotage IA n'est par construction pas concerné par l'explication
  « pilotage humain perfectible » avancée pour le corpus précédent : cette troisième mesure
  **affaiblit nettement** cette hypothèse comme cause principale et concentre le doute sur les deux
  autres — roll-out détecté trop large/précoce, ou borne haute `OK_PERFECT_GROOVE_TIME_MAX_S` à
  revoir (par type ou globalement) pour la géométrie de pattern réellement volée dans DCS. À trancher
  avant toute promotion de `_OK_`.

  **Mise à jour 7 septembre 2026 (corpus humain `Justice`, 6 recoveries, 6 septembre soir) — premier
  signal dans l'autre sens.** `groove_time_secs` connu sur 5 des 6 passes : **12,99 / 13,88 / 14,12**
  (groupe de 21h00) puis **20,08 / 21,88** (groupe de 23h15). Les trois premières valeurs sont **sous
  15 s** — jamais observé jusqu'ici sur aucun corpus (IA ou humain), qui n'avait montré que des
  valeurs au-dessus de 18 s. Casse l'hypothèse jusque-là dominante d'un roll-out "toujours détecté
  trop large/précoce" (qui prédirait un biais uniquement vers le haut) : ce même pilote, la même
  soirée, produit des groove times tantôt bien en dessous, tantôt bien au-dessus de la fenêtre
  15-18 s selon le groupe de passes (avant/après 23h). Aucune des 5 valeurs ne tombe dans la fenêtre.
  Piste à considérer : une différence de conditions entre les deux groupes (espacement de circuit,
  vent, fatigue/style de pilotage) plutôt qu'un biais systématique de détection dans un seul sens ;
  aucune conclusion encore possible sur le nombre de causes en jeu avec un échantillon aussi mélangé.
  Renforce simplement la conclusion déjà écrite : ne pas promouvoir `_OK_` avant d'avoir tranché.
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

- **Écart latéral quasi constant près du toucher — désormais reproduit sur F-14 *et* F/A-18, et
  identifié comme la cause directe du plafonnement à `--` de 6 passes sur 6 dans le test IA du 6
  septembre après-midi.** Initialement observé (~0,75-0,85 m, présent à toute distance dès 50 m) sur
  les 5 appontages d'un premier test live CVN-72 du 5 septembre 2026 (matin), réussis comme Cut, en
  reconstituant l'écart réel en mètres depuis `lineup_deg`/`distance_m` du JSON — l'écart ne grossit
  pas proportionnellement à la distance (ce qu'on attendrait d'une vraie erreur d'alignement ou d'un
  axe de référence mal orienté), ressemblant davantage à un décalage fixe de point de référence
  (crosse vs CG projetée, ou point de visée supposé) qu'à un vrai écart d'alignement croissant.
  **Le test IA du 6 septembre après-midi reproduit exactement ce comportement, cette fois sur les 6
  passes accrochées (4×F-14B(U), 2×F/A-18C) et avec un effet mesuré sur la note** : les trois gates
  ponctuelles (3/4, 1/2, 1/4 NM) sont toutes propres sur les 6 passes (`abs(GS) < 0,5°`,
  `abs(LU) < 1,0°`, largement dans la bande `OK`), mais `trajectory_deviations` montre un
  `lineup_deg` qui croît de façon lisse et quasi identique en pente sur les 6 passes, franchissant
  `LATE_WINDOW_LU_DEG = 1,5°` autour de 150-160 m et atteignant ~2,5° vers 80 m — ce qui plafonne
  chacune des 6 passes à `--` (2,0 pts) via la pondération temporelle de fin d'approche
  (`LATE_WINDOW_*`), malgré des gates par ailleurs dignes d'un `OK`. **Reproduit à l'identique sur
  F/A-18C**, qui ne porte aucune correction d'offset de crosse connue : ceci **affaiblit** l'hypothèse
  d'un offset de crosse F-14 mal calibré comme cause unique (voir `F14_HOOK_VERTICAL_CORRECTION_M`
  ci-dessous, qui reste une correction *verticale* distincte) et pointe plutôt vers soit un vrai
  comportement de pilotage IA DCS cohérent sur les deux modules (dérive légère et systématique vers
  un côté du centerline en courte finale), soit un point de référence géométrique partagé (axe
  BRC/angle de pont, plutôt qu'un offset par avion) commun aux deux calculs. Devient la piste de
  notation la plus prioritaire du projet : elle explique à elle seule pourquoi aucune des 6 passes IA
  de ce test n'a dépassé `--`, indépendamment des questions `_OK_`/temps de groove ci-dessus.
  `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` neutralise l'explosion `atan2` très près du pont, mais **pas**
  cet effet dans la fenêtre 80-160 m visée par `LATE_WINDOW_LU_DEG`, contrairement à ce qui était
  supposé avant ce test. Reste à faire : comparer le lineup au niveau de la trajectoire complète
  (pas seulement gates + fenêtre tardive) pour confirmer que l'écart est bien présent dès l'entrée en
  groove sur ce corpus IA (comme sur le corpus humain de matin) et non une dérive qui n'apparaît que
  tardivement ; et déterminer si un axe de référence (BRC vs angle de pont réellement utilisé) plutôt
  qu'un offset par avion explique la parenté F-14/F/A-18.

  **Mise à jour 6 septembre 2026 (corpus historique déposé par l'utilisateur,
  `.ignore/json-human-test/`, 300 rapports humains, 13 juin-23 août 2026, autre lignée du projet —
  schéma différent, pas `schema_version: 3`, code non comparable à l'actuel).** L'utilisateur précise
  que le module ayant produit ces rapports a évolué sur la période couverte : **les notes/grades
  qu'ils contiennent ne sont pas fiables comme repère de qualité et n'ont donc servi à rien dans
  l'analyse ci-dessous** — seule la géométrie brute (`gate_deviations`, `datums`) est exploitée, avec
  un simple seuil de plausibilité sur la magnitude (`|lineup| <= 100 ft` à 1/4 NM, ~3,7°, pour écarter
  les trajectoires clairement hors piste) plutôt qu'un tri par grade. Deux apports malgré
  l'incompatibilité de schéma : (1) une poignée de rapports (grades mis de côté) montrent malgré tout
  des écarts de lineup à 1/4 NM très modestes (9-22 ft, ~0,3-0,8°) sur des trajectoires par ailleurs
  cohérentes — indice qu'obtenir un écart de cet ordre **n'est pas physiquement impossible** dans ce
  simulateur/cette mission, indépendamment de la fiabilité des grades eux-mêmes qui les accompagnent.
  (2) Sur les 261 rapports portant un `gate_deviations.at_quarter_nm`, le signe de l'écart de lineup à
  1/4 NM se répartit quasi parfaitement également, **sans aucun filtre** : 126 négatifs / 135
  positifs (et 99/100 une fois les 62 valeurs `> 100 ft` écartées comme aberrantes) — **pas de biais
  unilatéral dominant** sur un corpus large et divers (nombreux pilotes, nombreuses sessions,
  plusieurs mois, plusieurs versions du logiciel), contrairement au corpus du 6 septembre après-midi
  (14 passes IA+humain, toutes du même côté). Le tout dernier point de trajectoire avant le toucher
  montre un léger débalancement (114 négatifs / 85 positifs, 57 %/43 %) mais rien de comparable au
  100 % unanime du 6 septembre. Cette analyse ne dépendant plus des grades, la conclusion est plus
  solide qu'une première passe (biaisée par un tri sur `pass_grade`, corrigée le même jour) : ceci
  affaiblit l'hypothèse d'un bug géométrique partagé (type `deck_angle`) **comme cause générale et
  permanente** — **mais cette lignée utilise un schéma JSON et vraisemblablement un calcul de
  géométrie entièrement différents du code actuel**, donc la comparaison reste indicative, pas
  probante pour le `deck_angle`/repère de lineup du code présent, et n'exclut pas un bug ou un effet
  de conditions spécifique à la session du 6 septembre (porte-avions/vent/cap/version de mission
  donnés) qui n'aurait simplement pas de raison d'apparaître dans un corpus produit à d'autres dates
  par un autre code. **La comparaison réellement décisive resterait à faire** : un corpus humain
  capturé avec le code *actuel* (`schema_version: 3`, ex. les 8 rapports du test humain du 5
  septembre soir déjà cités plus haut) permettrait de vérifier directement si le même biais
  unilatéral que le test IA du 6 septembre après-midi s'y retrouve déjà — à demander/retrouver avant
  de conclure.

  **Mise à jour 6 septembre 2026, fin d'après-midi — comparaison décisive obtenue, hypothèse du bug
  géométrique partagé affaiblie.** 4 passes humaines (`Justice-20260906`, même pilote, même
  session/génération, même porte-avions, code/commit identique au test IA de l'après-midi) donnent un
  lineup à 1/4 NM de **-1,27° / +0,62° / +0,28° / -0,37°** — **deux signes différents sur 4 passes
  consécutives du même pilote, sur le même porte-avions, avec le code actuel.** Si le biais unanime
  du 6 septembre après-midi (14 passes du même côté) venait d'une constante géométrique partagée
  (`deck_angle` ou repère de lineup, indépendante du pilote), ce nouveau corpus humain — même code,
  même navire — aurait dû montrer le même signe systématique ; ce n'est pas le cas. **Combiné à
  l'absence de biais unilatéral déjà observée sur le corpus historique (300 rapports, autre lignée),
  ceci pointe désormais plutôt vers une coïncidence propre au test IA du 6 septembre (pattern IA
  scripté/déterministe menant à une dérive répétable d'un vol à l'autre) que vers un bug de géométrie
  dans le code actuel.** Reste ouvert : pourquoi les 14 passes IA d'un même test convergent-elles
  autant entre elles, si ce n'est ni un bug de code ni une contrainte physique universelle — piste la
  plus probable désormais : le pilotage automatique DCS (IA) rejoue une même logique de correction
  d'un vol à l'autre dans des conditions quasi identiques (même vent, même carrier), ce qui produirait
  naturellement une dérive répétable sans qu'aucun bug LSO ne soit en cause. À vérifier sur un futur
  test IA avec un vent différent (sens ou force) : si le signe de la dérive suit le vent, c'est
  confirmé ; s'il reste identique quel que soit le vent, l'hypothèse d'un biais de pilotage IA
  indépendant du vent prendrait le relais de celle du bug géométrique.

  **Question posée par l'utilisateur (« le calcul peut-il seulement accorder un (OK)/`_OK_` ? »),
  désormais tranchée par la preuve, pas par la seule inférence** : la 3ᵉ passe de ce même corpus
  (`LSO-20260906-160913-...json`) obtient **`OkParentheses` (3,0 pts, lineup 1/4 NM = 0,28°, GS 1/4 NM
  = 0,34°)** avec le code/commit actuel. **Première confirmation live que le code présent peut bien
  produire mieux que `--`** — le doute exprimé plus haut ("0/14 passes vivantes n'ont jamais dépassé
  `--`") est levé pour `(OK)` au moins ; `_OK_`/`OK` restent, eux, toujours sans preuve live à ce
  jour.

  **Mise à jour 7 septembre 2026 — troisième corpus humain indépendant, même conclusion.** 6
  nouvelles passes du même pilote `Justice`, même soirée mais session/génération différente de celle
  ci-dessus (`.ignore/tests-20260906-human/`, hors dépôt Git) : lineup à 1/4 NM = **+1,60° / -3,00° /
  -1,48° / +3,20° / +0,35° / ~0,00°** — de nouveau les deux signes représentés (3 positifs, 2
  négatifs, 1 quasi nul), sur un troisième échantillon indépendant du même pilote/porte-avions. Ajoute
  du poids à la conclusion déjà écrite (coïncidence propre au test IA, pas un bug géométrique
  partagé) sans rien changer d'autre. À l'inverse, aucune des 6 passes ne dépasse `--` cette fois
  (toutes `NoGrade`/2,0 pts) : ne remet pas en cause la confirmation déjà obtenue que `(OK)` est
  atteignable, mais n'apporte rien de neuf pour `_OK_`/`OK`, toujours sans preuve live.

  **Sur la porte 3/4 NM** (voir "Décisions encore ouvertes" plus bas pour la décision elle-même) : le
  lineup à cette porte est négatif sur les 6/6 passes de ce corpus (-7,78° / -16,64° / -12,18° /
  -6,65° / -4,20° / -2,90°) — cohérent avec le constat déjà écrit ("7 passes sur 8" sur le corpus du 5
  septembre soir) que cette porte tombe structurellement dans le virage base-à-finale pour ce
  pilote/ce porte-avions, une reconfirmation de plus plutôt qu'un signal nouveau.

  **Mise à jour 7 septembre 2026, soir — nouveau test 100 % IA (8 unités, 4×F-14B(U) + 4×F/A-18C,
  CVN-72, `--ki`, `--position-source buffered`, `-vv`), signal le plus net obtenu à ce jour et
  probablement décisif contre l'hypothèse de coïncidence de pilotage.** 7 des 8 unités ont produit un
  rapport (6 posers + 1 remise de gaz `WaveoffUnknown`/`go_around_initiator_unknown` sans lineup
  exploitable) ; **les 6 posers plafonnent tous à `NoGrade`/`--` (2,0 pts)** malgré des portes 3/4,
  1/2 et 1/4 NM toutes propres (`abs(GS) < 0,5°`, `abs(LU) < 1,0°` partout, qualité `valid` sans
  exception) — la dégradation vient donc entièrement de la trajectoire continue, pas des gates. Sur
  les 6 posers **et** la remise de gaz (7/7), `groove_entry.lineup_deg` est négatif et resserré entre
  **-0,49° et -0,70°**, avec un `lineup_rate_deg_per_s` déjà négatif à l'entrée (-0,05 à -0,14°/s) —
  un avion en régime a priori stabilisé montre donc déjà la même dérive avant même le début du
  segment noté. Le lineup continue de croître ensuite de façon lisse et quasi identique sur les 6
  posers, franchissant `LATE_WINDOW_LU_DEG = 1,5°` vers 150-200 m et atteignant **-2,2° à -2,45° au
  dernier échantillon avant contact sur les 6/6**, ce qui plafonne chacune à `--` via la pondération
  temporelle de fin d'approche — reproduction quasi identique en forme et en signe du test IA du 6
  septembre après-midi, mais cette fois avec un F/A-18C **et** un F-14B(U) montrant la même valeur de
  convergence à quelques centièmes de degré près. **Point nouveau, potentiellement le plus important** :
  reconstitué en mètres (`distance_m * tan(lineup_deg)`), l'écart latéral réel décroît de façon
  monotone et cohérente sur les 6 passes — environ 12 m à 3/4 NM, ~6 m à 300 m, ~3,5 m à 100 m, moins
  de 0,5 m au dernier échantillon avant contact — ce qui ressemble à une **vraie convergence physique
  vers l'axe** (pas un offset fixe constant comme d'abord suspecté début septembre) ; c'est la
  conversion en degrés via une distance qui tend vers zéro qui manufacture l'essentiel de l'excursion
  visible dans `LATE_WINDOW_LU_DEG`, bien avant le plancher `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75 m`
  déjà corrigé début septembre — confirmant très concrètement, sur ce corpus, la piste déjà notée
  ("l'effet existe aussi dans la fenêtre 80-160 m visée par `LATE_WINDOW_LU_DEG`, non couverte par le
  plancher de 75 m"). Combiné au signe unanime et à la valeur resserrée dès l'entrée en groove sur 7
  appareils/2 types différents dans la même session, ceci pointe fortement vers soit (a) un axe de
  référence géométrique partagé (BRC/angle de pont) légèrement dévié de l'axe réellement utilisé côté
  DCS pour ce porte-avions/cette mission, soit (b) un comportement IA DCS réellement partagé entre
  modules sur cette mission précise (vent quasi identique au test du 6 septembre après-midi : `0°
  vrai / ~4,2-4,3 m/s`, donc ce test ne permet toujours pas de trancher via un vent différent comme
  proposé précédemment). **Reste à faire avant toute conclusion définitive** : (1) rejouer avec un
  vent nettement différent (sens et/ou force) pour voir si le signe/la magnitude de la dérive suit le
  vent ; (2) si possible un test humain sur cette même mission/porte-avions pour vérifier si un pilote
  humain montre la même convergence en mètres près du pont ou un profil différent ; (3) tracer
  explicitement l'écart en mètres (pas seulement en degrés) dans `trajectory_deviations` ou un outil
  de diagnostic dédié, pour ne plus avoir à le reconstituer a posteriori à chaque analyse.

  **Anomalie de vent près du niveau mer/pont reconfirmée sur ce même test** (voir aussi le point P1
  dédié plus haut) : sur les 7 rapports, la probe `wind_reference_probes.low` lit `180°/0,0 m/s`
  (au lieu de `0°/~4,2-4,3 m/s` cohérent avec la probe haute) sur 4 rapports sur 7, et la requête
  séparée de fin de rapport lit la même valeur aberrante sur 5 rapports sur 7 — nouvelle confirmation,
  sur un cinquième corpus indépendant, que l'anomalie touche spécifiquement les requêtes `GetWind`
  proches du niveau de la mer/du pont, sans jamais faire échouer le RPC lui-même
  (`wind_reference_established` reste `true` dans tous les cas).
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

- **Externaliser les seuils de tuning dans un fichier `lso.toml` placé à côté de l'exécutable.**
  Un brouillon documenté existe à la racine du dépôt, mais le binaire ne le charge pas encore et
  toutes les valeurs métier restent compilées dans Rust. Implémenter un chargement au démarrage
  avec `--config <chemin>` prioritaire, puis recherche par défaut à côté de `lso.exe` ; ne pas faire
  de hot reload dans une première version afin qu'aucune passe ne puisse changer de règles en
  cours de collecte. Réserver l'externalisation aux seuils `PROJECT-DERIVED` de détection d'entrée
  en groove, grading CATOBAR/V/STOL, Cut, correction/tendance et, dans des sections avancées
  clairement séparées, outcome/estimation de brin. Garder compilés les invariants de télémétrie
  (100/300/1 000 ms), distances des gates, géométries avion/navire, capacités mémoire, isolation et
  règles de sécurité. Le chargeur doit accepter les clés absentes via les défauts compilés, refuser
  clés inconnues, nombres non finis et ordres incohérents (`perfect <= OK <= no-grade`, fenêtre min
  <= max, Cut plus sévère que No Grade), et échouer explicitement au démarrage sur une configuration
  invalide plutôt que revenir silencieusement aux défauts.

  La configuration ne nécessite ni variable `schema_version` ni `profile` dans le TOML final ; ne
  pas confondre ce choix avec le `schema_version` du rapport JSON, qui reste obligatoire. À la
  création de chaque `Track`, figer un snapshot immuable de la configuration **effective après
  application des défauts**. Sérialiser ce snapshot complet dans le JSON avec un SHA-256 sémantique
  calculé sur une représentation canonique des valeurs effectives, et conserver séparément
  `grading_version`, version/commit/dirty du binaire et données brutes nécessaires au rejeu. Ne pas
  utiliser comme identité principale le hash des octets TOML : commentaires, espaces ou ordre des
  clés ne doivent pas créer artificiellement une nouvelle configuration métier. Journaliser au
  démarrage le chemin résolu et le hash, sans secret ni contenu ambigu.

  Étendre les diagnostics hors-ligne, en priorité `groove-ab`, pour accepter une ou plusieurs
  configurations et comparer leurs effets sur un même corpus sans modifier les fichiers d'entrée.
  Couvrir par tests : fichier absent et défauts historiques inchangés ; configuration partielle ;
  rejet de chaque incohérence ; résolution `--config`/dossier exécutable ; canonicalisation stable ;
  hash différent dès qu'une valeur effective change ; snapshot identique pendant toute une track ;
  séparation CATOBAR/V/STOL ; reproduction bit-à-bit des grades actuels avec les valeurs du
  brouillon. Mettre ensuite à jour AGENTS.md, CHANGES.md, primer.md et README.md lorsque le fichier
  deviendra effectivement consommé ; toute modification des seuils restera à revalider sur corpus
  puis en mission live selon sa portée.

- **Boucle de redémarrage pendant une pause/un rechargement de mission** — reconfirmé le 5 septembre
  soir : deux générations consommées sans log exploitable, puis deux générations de repli
  (`session_id` négatif) créées à 6 s d'intervalle sur `GetSessionId` en timeout jusqu'à
  rechargement complet, en cohérence avec l'erreur Lua `grpc.lua:288` côté DCS pendant le
  rechargement (voir aussi le point d'observabilité en P1 ci-dessus). Un backoff progressif
  réduirait le bruit de logs lors d'une pause prolongée.
- **Sémantique du ring à confirmer avec les nouvelles métriques.** Les anciens rapports montrent
  beaucoup d'overflow/capacity churn mais `lost_snapshots: 0`, sans rupture de séquence prouvée. Le
  JSON courant sépare maintenant évictions capacité/rétention, pertes réellement observées par le
  curseur, continuité du temps source et retard de livraison. Sur un prochain test `-vv`, comparer
  ces quatre familles avant toute conclusion sur le producteur ou le consommateur. Une amélioration
  du fork ne serait justifiée que si cette instrumentation prouve une ambiguïté restante ; aucune
  modification Lua/DCS-gRPC n'est autorisée ni nécessaire à ce stade.
- **Tentatives d'approche avortées avant le groove, invisibles hors logs DEBUG.** Déjà listé ;
  reconfirmé avec des chiffres concrets le 5 septembre soir : 3 faux départs de détection sur la
  soirée (un au catapultage T-45 lui-même : 55 s puis 2 s de flux bufferisé + échantillonnage crosse
  ouverts puis arrêtés sans motif visible ; deux autres sur un F-14, 52 s/8 s puis 42 s), pour un
  coût mesuré d'environ 2,5 min de flux inutiles et 5 cycles start/stop côté Lua sur la soirée.
  Piste additionnelle par rapport à la version précédente de ce point : exclure de l'armement du
  détecteur un avion qui vient de se trouver sur le pont (position proche de zéro, altitude proche de
  zéro, vitesse < 30 m/s) pendant une fenêtre de N secondes, ou exiger une vitesse verticale négative
  et une distance déjà positive pour armer `detect_recovery_attempt` (les deux motifs d'abandon
  sont déjà loggés en INFO avec durée écoulée et altitude minimale atteinte, voir
  [CHANGES.md](CHANGES.md) — reste ouvert : la compaction/exclusion elle-même n'est pas
  implémentée). **Mise à jour 6 septembre après-midi (test IA)** :
  6 abandons supplémentaires mesurés (27-100 s, altitude minimale systématiquement 220-241 m —
  jamais proche du pont) **sur un corpus 100 % IA**, ce qui nuance l'hypothèse initiale : ces faux
  départs ne sont pas spécifiques à un comportement humain (pattern overhead, hésitation) mais
  semblent structurels à la simple enveloppe géométrique du détecteur (3,5 NM/1100 ft) sur *tout*
  circuit Case I, humain ou IA — un avion en initial/break/vent-arrière traverse et quitte cette
  enveloppe plusieurs fois avant l'approche réellement notée. La piste "position proche de zéro"
  ci-dessus n'aurait rien changé à ces 6 cas précis (altitude minimale toujours > 220 m) ; une piste
  plus pertinente pour ce corpus serait d'exiger une tendance d'altitude décroissante sur la fenêtre
  d'armement plutôt qu'un simple seuil de position pont.
- **Sampler de crosse indépendant : erreurs RPC à expliquer et optimisation éventuelle.** Le second
  corpus humain du 7 septembre valide la capacité du ring : aucune troncature, y compris sur une
  track anormalement longue de 6 min 31 s, et les observations proches du contact sont conservées.
  La capture positions reste à 20 Hz et aucun échantillon hook n'est perdu par le canal, ce qui
  confirme l'isolation du chemin critique. En revanche, le log cumule 1 306 RPC hook en erreur
  `cancelled` pour 2 545 succès (zéro `deadline_exceeded`) ; les états T&G/bolter restent correctement
  déterminés grâce aux succès, mais la cause de ces annulations doit être isolée avant optimisation.
  L'activation seulement pendant groove/dernier quart reste ouverte : ne l'envisager qu'avec warm-up
  mesuré et sans perdre les transitions pré-groove utiles ni rendre le sampler dépendant du détecteur.
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
- **Taille des JSON de rapport** : `datums` et `hook_observation.timeline` dominent déjà les rapports
  de 0,94–1,50 Mo ; porter la timeline à 2 048 protège le diagnostic mais peut augmenter ce coût sur
  les longues sessions. Mesurer la taille sur un nouveau corpus avant d'envisager une compaction
  pré-groove ; toute compaction doit préserver transitions, erreurs et warm-up, sans dépendance
  fragile envers le détecteur de groove.
- **Script `run-live-buffered.ps1` non autonome pour un déploiement hors du poste de développement**
  (observé le 5 septembre soir, sur la machine de test qui n'a pas le lecteur `E:` du dépôt de
  développement) : exige `--baseline-manifest live-baseline.json` dans son propre dossier sans que ce
  fichier soit déployé avec le script (le rendre optionnel si absent, ou documenter qu'il doit être
  livré à côté) ; variable `$lsoRoot` calculée et jamais utilisée ; webhook Discord en clair dans le
  script **et** dans `start.bat` (à externaliser en variable d'environnement, comme pour le token
  gRPC). **Mise à jour 7 septembre 2026, soir** : le correctif d'encodage forcé du 7 septembre
  matin (`Tee-Object -Encoding utf8`) fait échouer le script au démarrage sur ce poste de
  développement lui-même — `Tee-Object` de Windows PowerShell 5.1 (celui réellement installé ici)
  n'a pas de paramètre `-Encoding` (ajouté seulement en PowerShell 7). Remplacé par
  `ForEach-Object { $_; Add-Content -LiteralPath $logFile -Value $_ -Encoding UTF8 }`, qui reste
  compatible 5.1 et force malgré tout un encodage UTF-8 cohérent (avec BOM, contrairement à
  `Tee-Object` par défaut qui écrit en UTF-16LE — cause d'origine du log mixte du 6-7 septembre).
  Non testé sur la machine de déploiement distincte mentionnée ci-dessous ; à vérifier si jamais
  cette machine utilise PowerShell 7 plutôt que 5.1 (le correctif fonctionne dans les deux cas, donc
  aucun risque de régression connu). Par ailleurs, le dossier `Records` de cette machine mélange des rapports `schema_version: 3`
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

- **Robustesse multi-recoveries simultanées — partiellement infirmé le 6 septembre après-midi.** Le
  test IA (8 unités, `--suspend-detectors-during-recovery`, nouveau compteur diagnostique
  `ActivePriorityPlanes::active_count`) a bien observé un chevauchement réel : F/A-18C-3-1 et
  F/A-18C-4-1 se sont retrouvés simultanément en `record_recovery` sur le même porte-avions à 4
  reprises (`concurrent_recoveries=2` en log), sans crash ni corruption croisée observée dans les
  rapports produits. **Portée limitée** : les deux occurrences concurrentes étaient des faux départs
  (jamais sous 100 m MSL), jamais deux approches complètes jusqu'au toucher en parallèle — la
  robustesse d'une vraie double recovery simultanée jusqu'à l'issue (deux JSON/ACMI/PNG publiés au
  même moment) reste donc à confirmer, mais l'isolation par paire tient au moins sous détection
  concurrente réelle.
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

**Reconfirmé le 6 septembre 2026 après-midi** : F/A-18C-3-1 a enchaîné des waveoffs en boucle
pendant le test IA (arrêt volontaire du test par l'utilisateur pour cette raison) ; les deux passes
capturées avant l'arrêt portent toutes les deux exactement `GRADE:WO WOFDIC` — le même code
DCS "Foul Deck Indicator Comment" que la session du 5 septembre matin. Signature identique, donc
même hypothèse : à vérifier côté pont/mission (zone d'appontage occupée, gear pas en batterie,
LSO déjà occupé par un autre appareil) avant de suspecter le module LSO — d'autant que la
"Robustesse multi-recoveries simultanées" ci-dessus montre justement F/A-18C-3-1 et F/A-18C-4-1 en
détection concurrente au même moment sur ce test, cohérent avec un encombrement de pont réel plutôt
qu'un artefact LSO.

## Décisions encore ouvertes (mission/serveur nécessaires)

Tous les points ci-dessous sont corrigés et testés unitairement (`cargo test`/`clippy`/`fmt`
propres au moment de leur implémentation, voir [CHANGES.md](CHANGES.md) pour le détail de chaque
changement), mais la preuve DCS live disponible reste partielle :

- **Segmentation d'un LQM `GRADE:WO` en tentative terminale indépendante** : le second corpus humain
  du 7 septembre a fusionné deux waveoffs puis un trap `WIRE# 2` dans une seule track de 6 min 31 s ;
  le premier LQM gagnait et les deux suivants devenaient `duplicate_ignored`, produisant finalement
  un faux `unconfirmed_arrest`. `Track::set_dcs_grading` établit désormais `WaveoffUnknown` dès le
  premier `GRADE:WO` matching et le garde de départ existant ferme la track quand la distance
  regrossit de plus de 150 m. Le test déterministe `WO -> WO -> WIRE# 2` vérifie trois états frais et
  conserve le brin 2 final. À revalider live : obtenir trois rapports distincts, chacun avec un seul
  LQM accepté, et vérifier que la courte queue conservée jusqu'au départ ne manque pas le début du
  circuit suivant.
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
- **`wire_estimate_at` n'accorde plus `confidence: "high"` sans brin confirmé par DCS** (implémenté
  le 6 septembre 2026, moitié du point P1 "estimation Rust du brin systématiquement décalée"
  ci-dessus). Une fenêtre de corrélation serrée (bracket/lag <=150 ms) prouve seulement une mesure
  précise du franchissement, jamais un arrêt réel — corrige le cas confirmé où un survol/bolter sans
  aucun accrochage produisait une confiance « high » sémantiquement fausse. Sans brin `WIRE#`
  confirmé par le LQM, la confiance plafonne désormais à `"medium"`. Reste à revalider en mission
  live que ce plafond ne masque pas, à l'usage, un cas où l'estimation Rust était en fait fiable
  malgré l'absence de confirmation DCS (ex. LQM injoignable) — actuellement aucun moyen pour un
  consommateur du rapport de distinguer ces deux causes de `"medium"` autrement qu'en lisant `reason`.
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
- **Détecteur d'entrée en groove stable-axis du 7 septembre : revalidation live partielle.** Les
  seuils courants (`|lineup| <=2°`, bank/route `<=10°`, pente de lineup `<=0,5°/s`, persistance
  `0,75 s`, gap max `300 ms`) ont été calibrés par replay exact de la géométrie `datums` sur les 12
  passes F-14B(U) du 7 septembre. Ils retardent notamment 09:24 de 23,19 à 17,39 s et 09:28 de
  21,49 à 17,14 s, sans uniformiser le reste du corpus (13,76–28,41 s sur les durées calculables).
  La passe 09:28 reste géométriquement `--` à cause d'un défaut tardif (`max |lineup|` 2,79°) : le
  détecteur ne blanchit donc pas cet écart. Le second corpus humain du 7 septembre le revalide sur
  trois passes F-14B(U) isolées : entrées à 756/368/346 m, lineup 0,25/-0,53/0,12°, durée stable
  exactement 0,75 s/16 samples, et replay `groove-ab` identique ; les lineups de virage antérieurs
  (-9,39/-21,50° à 3/4 NM, -7,53° à 1/2 NM) ne déclenchent plus prématurément. Le trap final n'entre
  justement pas en groove (lineup encore 3,51° à 1/2 puis 2,64° à 1/4), cohérent avec le `GRADE:C`
  DCS, mais sa track fusionnée empêche d'évaluer séparément les deux waveoffs précédents. Reste à
  vérifier avec un binaire propre : autres pilotes/types, vent de travers, correction tardive
  légitime et absence de faux négatif. Les seuils sont `PROJECT-DERIVED`; ne pas les desserrer ni
  forcer artificiellement 15–18 s à partir de ce seul corpus.
- `cargo audit` reste à exécuter dès qu'un outil autorisé est disponible localement (la CI l'exécute
  déjà).
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

  **Mise à jour 7 septembre 2026 (corpus humain `Justice`, 6 recoveries, 6 septembre soir)** : les
  deux seules arrestations confirmées de ce corpus (`WIRE# 1` les deux fois) ont **toutes les deux**
  été jugées `GRADE:C` par le LQM DCS avec le commentaire `_TMRDAR_` ("too much rate of descent") —
  troisième et quatrième points de calibration humaine sur ce seuil, dans la continuité du signal déjà
  noté le 5 septembre soir. Sink rate max Rust observé sur ces deux passes : **6,40 m/s** (80 % du
  seuil, `LSO-...-t3695050.json`) et **8,00 m/s** (100 % du seuil pile, `LSO-...-t11757470.json`) —
  ni l'un ni l'autre n'a déclenché le Cut `dangerous_sink_rate_or_bank` (les deux notées `NoGrade`/
  2,0 pts, pas `C`/0,0 pts), très probablement faute des `>=3` échantillons consécutifs requis à
  l'intérieur du 1/4 NM plutôt que d'un seuil jamais atteint. **Quatre points de calibration humaine
  sur quatre pointent désormais dans le même sens** (5 septembre soir : 6,0 m/s jugé fautif par DCS,
  Cut Rust resterait silencieux ; ici : deux nouveaux cas TMRDAR à 6,4 et 8,0 m/s, Cut toujours
  silencieux) : DCS semble sanctionner un sink rate excessif nettement avant que notre seuil/garde de
  persistance ne s'active. Ne change rien à la garde anti-faux-positif (toujours aucun cas où notre
  Cut se serait déclenché à tort), mais renforce l'hypothèse que `SINK_RATE_CUT_MPS = 8,0 m/s` et/ou
  l'exigence de 3 échantillons consécutifs sont trop permissifs par rapport au jugement LSO DCS —
  décision à prendre avec l'utilisateur avant tout resserrement (aucune base NATOPS chiffrée pour
  arbitrer, voir plus haut).
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
