# Tasking & roadmap — DCS-gRPC-lso

> Idées, choix techniques ouverts, arbitrages à faire et bugs connus non résolus ou non revalidés.
> Ne contient que des points **encore en suspens** — dès qu'un point est résolu et revalidé (ou
> tranché sans besoin de revalidation), il est retiré d'ici ; l'essentiel synthétique de ce qui a
> été fait migre vers [CHANGES.md](CHANGES.md) (et [AGENTS.md](AGENTS.md) si ça touche l'état
> durable du système) — voir [AGENTS.md](AGENTS.md), "Règles de maintenance des documents markdown
> racine", pour la règle complète. Pour le détail narratif des sessions passées (dates, tests,
> discussions de conception), `git log`/`git show` sur les commits correspondants fait foi ; ce
> document ne le duplique plus. Dernière purge : 6 septembre 2026.

## À faire en priorité (P0)

Aucun point P0 ouvert.

## À investiguer — pas encore un bug confirmé (P1)

- **Écart latéral quasi constant (~0,75-0,85 m) présent à toute distance**, observé sur les 5
  appontages d'un test live CVN-72 du 5 septembre 2026 (soir), réussis comme Cut. Repéré en
  reconstituant l'écart réel en mètres depuis `lineup_deg`/`distance_m` du JSON : l'écart latéral ne
  grossit pas proportionnellement à la distance (ce qu'on attendrait d'une vraie erreur d'alignement
  ou d'un axe de référence mal orienté) — il reste quasiment constant de 50 m jusqu'au toucher, sur
  les 5 appontages sans exception. Ressemble davantage à un décalage fixe de point de référence
  (crosse vs CG projetée, ou point de visée supposé) qu'à un vrai écart d'alignement croissant.
  N'affecte pas la note de façon significative (`NEAR_TOUCHDOWN_ANGLE_REFERENCE_M` le neutralise
  déjà en grande partie) et n'a donc aucune urgence, mais mérite d'être creusé séparément si
  l'occasion se présente — cause non identifiée à ce stade.
- **Possible biais systématique de -1 brin dans l'estimation Rust du câble**, observé le 5 septembre
  2026 (test CVN-72 du soir). Sur 5 appontages complets, 2 divergences `wire_estimated`/`wire_dcs`
  (F14-4-1 : Rust 1 vs DCS 2 ; F18-2-1 : Rust 2 vs DCS 3) — dans les deux cas, l'estimation Rust
  (`continuous_hook_plane_crossing`) est exactement un brin en dessous de la valeur DCS/LQM.
  Seulement 2 échantillons, insuffisant pour conclure à un biais systématique plutôt qu'une
  coïncidence, mais le garde-fou existant (`wire_primary: dcs_lqm`, jamais l'estimation Rust
  affichée au pilote en cas de divergence) a fonctionné correctement dans les deux cas — aucun
  risque utilisateur immédiat, seulement une piste de précision à vérifier sur plus de données (un
  décalage constant suggérerait un franchissement de brin détecté en retard plutôt qu'un bruit
  aléatoire).
- **Déclin de l'AoA corrigée dans le groove** (~1,5-3° entre ¾ NM et ¼ NM, systématique sur 3-4 F-14
  dans un test avec vent nul). Le vent nul dans cette mission exclut un artefact de la correction
  vent introduite pour l'AoA ; reste à savoir si c'est un comportement de pilotage IA réel ou un
  effet de la décomposition en repère avion. Nécessite un outil de comparaison ancienne/nouvelle
  formule sur les mêmes données.
- **Délai fixe touchdown → fichiers complets** (~10,2 s, ±0,03 s observé) — trop régulier pour être
  du temps de calcul variable ; sent le délai intentionnel dans le pipeline (confirmation finale du
  brin ? cadence de lecture du buffer côté fork ?). Source exacte non identifiée.
- **Mémoire du process `DCS_server` en légère hausse continue** (+8,3 Mo de working set en ~6 min
  sur une session de test, jamais redescendue). Suivi démarré en cours de session (pas de baseline
  dès le lancement du serveur, ni de corrélation fine appontage-par-appontage). Insuffisant pour
  conclure à une vraie fuite mémoire côté DCS (pas de preuve qu'un appontage la libère jamais, mais
  pas non plus assez de recul pour l'affirmer). LSO n'écrit rien côté DCS et ne peut qu'observer
  depuis l'extérieur (compteur Windows du process) ; à refaire sur une session dédiée, plus longue,
  avec mesure dès le démarrage du serveur.

## Optimisations à considérer (P2)

- **Boucle de redémarrage pendant une pause mission** (observé : 6 redémarrages de génération en
  ~30 s, un par timeout de session ID). Sans gravité mais bruyant ; un backoff progressif réduirait
  le bruit de logs lors d'une pause prolongée.
- **Vent nul non exercé en test** : la mécanique de capture (2 requêtes `AtmosphereService.GetWind`
  à l'entrée du groove) a été validée mécaniquement, mais le calcul de correction lui-même (la
  soustraction du vecteur vent) n'a jamais été vraiment exercé avec du vent réel. À refaire avec du
  vent configuré dans la mission de test.
- **Purge du ring Lua sur `after_sequence` acquitté** et **`telemetryObservationErrors` bornée à 128
  entrées** : implémentées côté fork, à revalider en usage prolongé (pas un doute sur le principe,
  seulement sur l'absence de régression en usage long).
- **Tentatives d'approche avortées avant le groove, totalement invisibles hors logs DEBUG.**
  Observé le 5 septembre 2026 : un avion ayant déclenché une détection de pattern réelle
  (`found pattern / recovery attempt`) puis avorté (`discard as plane was never below 100m MSL`) ne
  laisse aucune trace dans le JSON, SQLite ou le tableau de bord. Correct sur le principe (rien à
  noter tant que l'avion n'est jamais descendu assez bas), mais un avion qui enchaîne les tentatives
  avortées (deck foul suspecté côté mission, voir note opérationnelle ci-dessous) n'apparaît nulle
  part dans les artefacts destinés au pilote/LSO humain — seul `-v`/`-vv` en direct le montre. Piste :
  un compteur minimal (nombre de tentatives détectées puis avortées avant groove, par avion/session)
  exposé quelque part de plus visible que le log DEBUG, sans fabriquer de rapport pour une approche
  qui n'a jamais existé au sens du grading.

## Hors-scope confirmé (rappel volontaire)

- **Robustesse multi-recoveries simultanées** : la dernière mission de test était trop minimaliste
  (8 unités, jamais deux avions en approche en même temps) pour être testée.
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

**Foul deck suspecté côté mission, pas côté LSO (5 septembre 2026).** En fin de session, un avion a
enchaîné 3 waveoffs consécutifs coupés systématiquement à ~165 m du pont (`WO?`, DCS `GRADE:WO
WOFDIC` à chaque fois), et un autre n'est jamais descendu sous 100 m MSL sur ses deux tentatives
(voir P2 ci-dessus). Schéma cohérent avec un pont indisponible (gear pas en batterie, zone
d'appontage occupée) plutôt qu'avec un problème de pilotage IA répété. DCS-gRPC n'expose aujourd'hui
aucun signal d'état du pont/de la passerelle arrière ; LSO ne peut donc ni confirmer ni infirmer
cette hypothèse et documente fidèlement "remise de gaz, initiateur inconnu" à chaque fois, comme
prévu par conception (voir [AGENTS.md](AGENTS.md), "jamais inventer OWO/WOP/waveoff pilote"). À
vérifier directement dans la mission/le jeu lors d'une prochaine session, pas par une modification
de LSO.

## Décisions encore ouvertes (mission/serveur nécessaires)

Tous les points ci-dessous sont corrigés et testés unitairement (`cargo test`/`clippy`/`fmt`
propres au moment de leur implémentation, voir [CHANGES.md](CHANGES.md) pour le détail de chaque
changement), mais **aucun n'a de preuve DCS live confirmant l'absence d'effet de bord** :

- Revalider en mission live `NEAR_TOUCHDOWN_ANGLE_REFERENCE_M = 75 m` : vérifié par reconstruction
  manuelle contre 5 rapports JSON d'un test live, mais jamais rejoué en conditions live pour
  confirmer qu'il replace bien en `Ok`/`(OK)` les passes affectées par l'ancien artefact et retire
  le Cut artefactuel correspondant, sans introduire de nouveau faux négatif (un écart réellement
  dangereux et bref, développé uniquement dans les tout derniers mètres, pourrait en théorie être un
  peu moins vite détecté qu'avant). 75 m n'a par ailleurs aucune base doctrinale chiffrée : choisi
  par cohérence physique (~1 s de vol à l'approche), pas calibré sur un corpus large.
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
- Revalider en mission live le correctif de la remise de gaz en survol (`DECK_CROSSING_ALT_CAP_FT`).
- Revalider sur données live les nouveaux seuils de notation (`PERSISTENCE_MIN_CONSECUTIVE_SAMPLES`,
  `OSCILLATION_MIN_SWING_DEG`/`OSCILLATION_MIN_REVERSALS`) : jamais rejoués sur un corpus de
  rapports live pour vérifier qu'ils ne masquent pas de vrais écarts ni ne déclenchent de faux
  positifs sur des approches réelles.
- Revalider sur données live le raffinement d'entrée en groove CATOBAR
  (`GROOVE_ROLLOUT_MAX_BANK_DEG`/`GROOVE_ROLLOUT_MAX_TRACK_ANGLE_DEG` = 15° chacun) : testé
  unitairement seulement (approches synthétiques). Reste à confirmer que 15°/15° ne retarde jamais
  une vraie entrée en groove sur un virage large ou un fort vent de travers, et qu'ils excluent bien
  un survol transitoire de la boîte pendant le virage 180→90 sur un enregistrement réel.
- `cargo audit` reste à exécuter dès qu'un outil autorisé est disponible localement (la CI l'exécute
  déjà).
- Revalider sur des enregistrements live précis les correctifs de la désynchronisation
  `wire_estimated`/`wire_estimation` et de l'explosion `atan2` de `trajectory_deviations` : corrigés
  et testés unitairement seulement, jamais rejoués sur les rapports live où les deux anomalies
  avaient été observées, pour confirmer qu'elles disparaissent bien sans effet de bord.
- Revalider sur données live le Cut sink-rate/bank-angle (`SINK_RATE_CUT_MPS = 8.0 m/s`,
  `BANK_ANGLE_CUT_DEG = 30°`) : contrairement à la plupart des autres seuils de ce module, celui-ci
  n'a **aucune base doctrinale chiffrée** à valider — seulement à confirmer, sur un corpus de
  rapports live variés (dont au moins une approche par gros temps/vent de travers, où le sink
  rate/bank réels sont naturellement plus élevés), qu'il ne déclenche jamais de faux positif sur un
  poser normal ni ne manque un vrai cas dangereux. Priorité avant toute promotion : le risque de
  faux positif est le plus élevé de tous les seuils du module, faute de nombre NATOPS à recaler
  dessus. **Premier contrôle fait le 5 septembre 2026** contre 9 rapports d'un test live (6 posés +
  3 remises de gaz) : aucun faux positif, et pas même un cas proche du seuil (sink rate maximum
  observé 6,75 m/s, 84 % du seuil ; bank maximum 3,5°, 12 % du seuil). Mais ce corpus est
  entièrement piloté par IA, avec des approches anormalement lisses : ce contrôle prouve l'absence
  de faux positif sur ce corpus précis, pas la justesse du seuil pour un vrai cas dangereux. Reste
  entièrement à faire : un enregistrement avec un pilotage plus agressif ou dégradé (humain, vent
  fort, correction tardive) pour tester si le seuil déclenche au bon moment.
- Revalider en mission live l'automatisation de `_OK_` (bande d'amplitude MOOSE-inspirée + fenêtre
  de temps de groove NATOPS 15-18 s) : deux points distincts à confirmer séparément. (1) La bande
  d'amplitude resserrée n'a, comme le Cut sink-rate/bank, aucune base NATOPS chiffrée — à vérifier
  qu'elle ne déclenche jamais `_OK_` à tort sur un poser simplement propre mais pas réellement
  parfait. (2) La fenêtre de temps 15-18 s, elle, est bien `OFFICIAL`, mais appliquée aujourd'hui
  identiquement au T-45 malgré sa pente différente (3,0° vs 3,5°) — à vérifier sur des posers T-45
  réels si cette fenêtre convient à son profil de vitesse propre ou si elle exclut à tort des T-45
  par ailleurs parfaits (ou, à l'inverse, en admet qui ne le seraient pas dans un découpage NATOPS
  pensé pour un jet standard). Jamais observé sur un vrai `_OK_` en conditions live — d'autant plus
  incertain que `_OK_` est, par conception, censé être rare ("Unicorn").
- Confirmer en mission live que les événements DCS pertinents (contact/`Land`, LQM) se comportent
  pour l'AV-8B/Tarawa comme pour les porte-avions CATOBAR : seule la géométrie/les seuils V/STOL
  (voir [VSTOL.md](VSTOL.md)) ont été travaillés en détail, la sémantique événementielle côté Tarawa
  n'a jamais été spécifiquement vérifiée en direct.
- Confirmer en mission live la polarité de la crosse (up/down) pour le T-45 (index de draw argument
  25, partagé avec le F/A-18C) et pour le F-14 (index 1305, toutes variantes) : seul l'index a été
  fourni, la convention `<= 0,2` = up / `>= 0,8` = down reste une hypothèse reprise du F/A-18C,
  jamais vérifiée indépendamment pour ces deux types. Priorité : réaliser un vrai touch-and-go
  volontaire (crosse relevée) sur un T-45 puis sur un F-14 en session live, et vérifier dans le JSON
  (`hook_observation.timeline`, valeurs `raw`) que la lecture bascule bien vers `<= 0,2` au moment
  voulu — sinon la polarité doit être inversée ou recalibrée pour le(s) type(s) concerné(s). Le
  corpus F/A-18C ayant servi à calibrer la convention actuelle (4 passes T&G + 1 arrêtée) montre une
  valeur brute stable à 0 avant contact puis une transition stable vers 1 : en répétant l'exercice
  sur T-45/F-14, vérifier que c'est bien le fil complet avant contact qui pilote la conclusion, pas
  un seul échantillon isolé après coup.
- Politique de skew pendant que le porte-avions vire ou accélère : le contrat de télémétrie
  (100/300 ms) n'a jamais été spécifiquement exercé avec un porte-avions en virage ou en
  accélération plutôt qu'en ligne droite/vitesse constante.
- Comportement V/STOL réel non simulé : palier vertical (VL) et roulé (RVL) réels, rebond/double
  contact — seule la classification (contact suivi d'un départ → neutre, jamais `Bolter`) a été
  pensée en amont ; aucune de ces situations n'a été observée sur un vrai vol Tarawa.
- Géométrie exacte d'occupation du spot 7.5 (AV-8B/Tarawa) jamais vérifiée en conditions réelles,
  seulement calculée depuis les références géométriques de [VSTOL.md](VSTOL.md).
- Aucun manifeste `--baseline-manifest` n'a jamais été réellement rempli/authentifié contre un
  déploiement live : build DCS, SHA-256 du DLL et des fichiers Lua déployés, versions de module
  avion restent à ce jour "non authentifiés" pour toute session live passée.

## Pistes de notation encore ouvertes

- **NC vs statut neutre dédié** : `grading_availability` (`available` / `unavailable_technical` /
  `unavailable_event_outcome`) distingue déjà, en JSON, une `NC` d'origine télémétrique d'une `NC`
  d'origine événementielle ; introduire un nouveau statut *pass_grade* dédié à "neutre par
  construction" (au lieu de fusionner les deux sous l'étiquette `NC` visible pilote) serait une
  décision de nommage/UX (quel libellé, quel impact sur le greenie board existant) plutôt qu'un
  correctif de clarté à faible risque — hors scope d'une implémentation automatique sans validation
  produit. Laissé ouvert pour une session dédiée si le besoin se confirme.
- **AoA réellement lue depuis le cockpit** (`aircraft_draw_argument`) : abandonné, pas un refus
  définitif. Le draw argument correspondant a été confirmé **absent** du modèle 3D pour au moins le
  T-45C (vérifié via ModelViewer 2.0 contre l'installation client) ; aucune source primaire
  vérifiable n'a été trouvée pour les trois autres avions non plus. Ne pas relancer cette piste sans
  accès DCS live pour un balayage empirique de `UnitService.GetDrawArgumentValue`, ou une nouvelle
  vérification ModelViewer pour les types restants. Remplacé pour l'instant par la correction du
  vent sur l'approximation géométrique existante (voir [CHANGES.md](CHANGES.md)).

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
5. hook argument à travers groove/finale pour chaque module CATOBAR supporté (F/A-18C, T-45, F-14) ;
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
