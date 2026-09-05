# Comment la note d'un appontage CASE I sur CVN est calculée, expliqué simplement

> Ce document explique, sans jargon de programmation, le raisonnement que suit le programme pour
> transformer une trajectoire d'appontage en une note (`OK`, `(OK)`, `--`, `C`, `B`, `WO?`, `NC`).
> Il se concentre sur le cas standard : porte-avions CATOBAR (Nimitz/Forrestal), approche CASE I de
> jour. Toutes les valeurs numériques sont exactes à la date de rédaction, mais restent des
> **règles du projet**, jamais une certification USN/USMC officielle — voir la mention
> "PROJECT-DERIVED" plus bas.
>
> Pour le "comment le programme observe l'avion" (repérage, enregistrement, événements), voir
> [primer.md](../primer.md). Pour la spécification technique exhaustive et à jour, voir
> [GRADING_REFERENCE.md](GRADING_REFERENCE.md). Ce document-ci n'est qu'un résumé pédagogique ; en
> cas de différence, GRADING_REFERENCE.md fait foi.

## L'idée en une phrase

Le programme regarde si l'avion reste bien centré sur la pente d'approche idéale (ni trop haut ni
trop bas) et bien aligné dans l'axe du pont (ni trop à gauche ni trop à droite), du moment où il
rentre dans le "groove" (l'axe final avant le pont) jusqu'au toucher. Plus l'écart est grand, et
plus il est grand *près du pont*, plus la note baisse. Un certain nombre de compléments plus fins
(vitesse de correction, oscillations, endroit de l'appontage) viennent ensuite affiner la note,
mais uniquement en **la baissant**, jamais en l'améliorant.

## D'où vient le principe général

Trois documents officiels NAVAIR fournissent le **vocabulaire** (les symboles `OK`, `(OK)`, `--`,
`C`...) et l'**esprit** de la notation LSO réelle. Mais ils ne fournissent pas de formule précise :
la façon exacte de convertir "l'avion est à tel endroit" en "telle note" est une reconstitution du
projet, jamais une règle officielle appliquée telle quelle. C'est pour ça que ce document répète
souvent "règle du projet" : c'est une notation d'entraînement crédible, pas un LSO certifié.

## Étape 1 — Mesurer l'écart en continu, tout le long du groove

Le programme n'attend pas des instants précis pour regarder l'avion : il enregistre sa position
par rapport au porte-avions à chaque frame, du moment où il entre dans l'axe final (le "groove")
jusqu'au toucher du pont. À chaque instant enregistré, il compare l'altitude et la position
latérale réelles de l'avion à ce qu'elles devraient être pour une pente d'approche standard, et en
déduit deux écarts angulaires :

- un écart de **pente** (glideslope) : l'avion est-il trop haut ou trop bas par rapport à la pente
  idéale, vu depuis le pont ?
- un écart d'**alignement** (lineup) : l'avion est-il trop à gauche ou trop à droite de l'axe du
  pont ?

C'est cette trajectoire continue, prise dans son ensemble, qui fournit le pire écart de pente et le
pire écart d'alignement de toute l'approche — le point de départ de la note (étape 3).

**Comment le programme sait-il que le groove commence (CVN, CATOBAR uniquement) ?** Dans la
doctrine réelle, ce n'est pas une distance ni une altitude précises : c'est le moment où le pilote
"roule ailes à plat, aligné sur l'axe du pont, boule centrée", après son virage final — un geste,
pas un seuil géométrique. Le programme ne peut pas observer ce geste directement, donc il s'en
approche avec ce qu'il peut mesurer : il exige que l'avion soit déjà dans une zone large (à moins de
¾ NM, moins de 300 ft, à peu près dans l'axe), **et**, une fois dans cette zone, que deux mesures
consécutives montrent à la fois un roulis quasi nul (l'avion ne tourne plus) et une trajectoire déjà
orientée vers le pont (pas seulement une position momentanément proche de l'axe pendant un virage
qui continue). Sans cette deuxième vérification, un avion qui coupe la boîte en pleine fin de virage
pourrait être compté comme "entré dans le groove" un peu trop tôt. Ce raffinement ne s'applique qu'au
CATOBAR (Nimitz/Forrestal) ; le V/STOL (AV-8B sur le Tarawa) n'a pas ce virage final à distinguer et
garde la zone large seule.

**Un piège dans les tout derniers mètres, corrigé le 5 septembre 2026.** Le programme calcule
l'écart de pente et d'alignement comme un *angle* vu depuis le pont — logique tant que l'avion est
encore loin, exactement comme un LSO humain juge une approche. Mais juste avant de toucher, un
avion fait toujours un léger "arrondi" (il se stabilise pour se poser en douceur), ce qui déplace sa
hauteur réelle de quelques décimètres seulement par rapport à la pente idéale — un écart minime et
normal. Le problème : plus l'avion est proche du pont, plus ce même petit écart, une fois transformé
en angle, paraît énorme (un peu comme un trou dans la route qui semble insignifiant vu de loin mais
immense quand on est juste au-dessus). Un test en conditions réelles le 5 septembre 2026 a montré
que deux appontages par ailleurs excellents (l'un d'eux noté "passe parfaite" par le jeu lui-même)
recevaient quand même la plus mauvaise note du programme, uniquement à cause de cet effet dans les 4
à 8 derniers mètres. Le calcul a été corrigé : dans cette toute dernière zone, le programme continue
de mesurer l'écart réel en mètres, mais ne le convertit plus en angle en le divisant par la distance
qui reste (de plus en plus petite) — il le compare plutôt à une distance de référence fixe, comme s'il
mesurait toujours depuis le même point. Un vrai problème (un écart de plusieurs mètres) continue donc
à faire baisser la note comme avant ; seul l'arrondi normal et attendu ne fait plus paniquer le calcul.

## Étape 2 — Trois repères précis dans cette même trajectoire : les "portes"

En plus de la lecture continue, le programme retient trois instants particuliers, à des distances
exactes du porte-avions, qu'on appelle des "portes" (gates) — ce sont les distances traditionnellement
utilisées pour juger une approche (3/4 NM, 1/2 NM, 1/4 NM) :

| Porte | Distance |
|---|---:|
| 3/4 NM | ≈ 1 389 m |
| 1/2 NM | ≈ 926 m |
| 1/4 NM | ≈ 463 m |

Ces trois repères servent à deux choses que la trajectoire continue seule ne garantit pas :

- ils sont validés avec une exigence de fiabilité renforcée (deux points de mesure encadrant
  précisément l'instant du passage, pas trop de retard entre les deux, temps qui avance
  normalement) — c'est le socle sur lequel repose la disponibilité même d'une note ;
- il en faut **trois, valides et dans l'ordre**, pour qu'une note favorable soit possible du tout ;
  sans ça, l'appontage est classé "non noté" (`NC`), quelle que soit la qualité de la trajectoire
  continue par ailleurs.

Autrement dit : la trajectoire continue capture *tout* ce qui se passe pendant le groove et fournit
la mesure la plus complète des écarts, mais ce sont toujours les trois portes, avec leur propre
exigence de fiabilité, qui décident si l'appontage est *notable* au départ. Une garde anti-"faux
positif" s'applique en plus à la lecture continue : une seule mesure isolée et aberrante (un pic dû
à un décrochage ponctuel de télémétrie, par exemple) ne compte pas toute seule — il faut au moins
deux mesures consécutives dans la même direction pour qu'un écart de la trajectoire continue soit
retenu. Cette prudence ne s'applique jamais à la sécurité "Cut" (voir plus bas) ni à la pondération
de fin d'approche (voir plus bas) : sur ces deux points précis, une seule mesure dangereuse suffit à
agir immédiatement, volontairement.

Au final, le pire écart trouvé sur l'ensemble de la trajectoire continue et le pire écart trouvé
sur les trois portes sont combinés en gardant le plus défavorable des deux — la trajectoire
continue **ne peut jamais améliorer** la note par rapport aux trois portes seules, elle ne peut que
confirmer ou l'aggraver (par exemple un écart survenu juste entre deux portes, que les trois
photos seules auraient manqué).

## Étape 3 — La grille de note de base

Une fois le pire écart de pente et le pire écart d'alignement connus (portes + trajectoire
continue combinées), la note de base est lue dans cette grille :

| Note | Condition | Points |
|---|---|---:|
| `_OK_` (passe parfaite) | `OK` (ci-dessous), en plus resserré (voir "Étape 3 bis"), avec un temps de groove de 15 à 18 secondes | 5,0 |
| `OK` | écart de pente < 0,5° et écart d'alignement < 1,0° sur toute l'approche | 4,0 |
| `(OK)` | écart de pente ≥ 0,5° ou écart d'alignement ≥ 1,0°, mais pas encore franchement gênant | 3,0 |
| `--` | écart de pente ≥ 1,0° ou écart d'alignement ≥ 2,0° | 2,0 |
| `C` (Cut, sécurité) | trop bas (pente en dessous de -2,5°) à la porte 1/4 NM, ou à tout moment de la trajectoire continue à 463 m du pont ou plus près ; ou taux de descente/gîte franchement dangereux et **soutenu** (≥ 3 mesures d'affilée) dans cette même zone | 0,0 |
| `B` (Bolter) | l'avion touche le pont mais repart sans s'arrêter, alors que les trois portes étaient valides | 2,5 |
| `WO?` | remise de gaz observée, mais impossible de prouver qui (pilote ou LSO) l'a déclenchée | pas de points |
| `NC` | télémétrie insuffisante/invalide, ou trap non confirmé | pas de points |

Le `C` (Cut) est une alerte de sécurité pure : dès qu'une seule mesure, même isolée, montre l'avion
trop bas tout près du pont, la note tombe immédiatement à `C`, sans attendre confirmation. C'est le
seul cas où une mesure unique suffit à trancher pour l'écart de pente, volontairement — c'est le
genre d'écart qui, dans la réalité, peut être dangereux.

**Un deuxième déclencheur du `C`, ajouté le 5 septembre 2026** : un taux de descente ou une gîte
(inclinaison) franchement dangereux, **soutenu sur au moins trois mesures d'affilée** (pas un pic
isolé — la garde est ici plus stricte que partout ailleurs dans le document, précisément parce
qu'un `Cut` retire tous les points), à cette même distance du pont. Contrairement au seuil de pente
ci-dessus, **ce seuil n'a aucune base chiffrée dans la doctrine officielle** : les manuels LSO
mentionnent bien un taux de descente ou une gîte excessifs comme des dangers réels ("too much rate
of descent", appel "level your wings"), mais ne donnent jamais de nombre — c'est un choix du
projet, assumé comme tel, et pas encore éprouvé sur un cas réellement dangereux en mission (voir
`tasking-roadmap.md`).

## Étape 3 bis — `_OK_`, la passe parfaite (ajouté le 5 septembre 2026)

`_OK_` n'est jamais un chemin séparé vers la note : il ne peut être atteint que depuis une passe
qui aurait de toute façon déjà mérité `OK` par tout ce qui précède (grille de base **et** les
compléments de l'étape 4 ci-dessous — tendance, oscillations, proximité du pont). C'est un
resserrement de `OK`, jamais une note obtenue autrement. Deux conditions, toutes les deux
nécessaires :

1. **Une fenêtre encore plus étroite que `OK`.** Chaque porte et chaque instant de la trajectoire
   continue doit rester dans une bande d'environ deux fois plus stricte que celle d'`OK` simple
   (par exemple ±0,3-0,4° de pente au lieu de 0,5°, ±0,5° d'alignement au lieu de 1,0°). Les trois
   portes n'ont droit à aucune exception, même une seule mesure limite les en exclut — elles sont
   déjà des mesures vérifiées et fiables. La trajectoire continue garde, elle, le même pardon
   anti-bruit qu'ailleurs dans ce document (étape 2) : une seule frame isolée et non répétée ne
   suffit pas à exclure la perfection, il en faut au moins deux d'affilée.
2. **Un temps de groove de 15 à 18 secondes**, la durée que le manuel officiel décrit pour un
   groove standard (voir plus bas). Si ce temps n'est pas connu, `_OK_` n'est jamais accordé —
   le programme ne suppose jamais qu'une preuve manquante aurait été favorable.

`_OK_` ne dépend **jamais** du brin accroché (1, 2, 3 ou 4 donnent exactement le même résultat à
écart et temps égaux) ni d'un touch-and-go volontaire, qui plafonne systématiquement un cran plus
bas, à `OK` au mieux, jamais `_OK_` — un touch-and-go est un exercice de qualification délibéré,
jamais un vrai arrêt complet, et "passe parfaite" est réservé à un vrai brin accroché.

**D'où viennent ces deux critères ?** Le symbole `_OK_` lui-même, et sa signification ("passe
parfaite"), sont bien officiels (manuel LSO américain, section 11.4.1). Le temps de groove de
15-18 secondes est aussi un texte officiel réel (manuel des procédures porte-avions, section
6.2.4.3) — mais aucun des deux manuels ne précise comment combiner ces informations en une règle
de calcul automatique ; c'est un choix du projet d'utiliser ce temps comme condition. La fenêtre
d'écart resserrée, elle, n'a aucune origine officielle : elle est reprise telle quelle d'un mod de
notation LSO open-source existant, utilisé par d'autres dans la communauté de simulation, qui a
d'ailleurs déjà inspiré plusieurs des seuils numériques présents ailleurs dans ce document. Une
ancienne règle locale liait aussi `_OK_` à un brin précis (le brin 3) et à une fenêtre de temps
légèrement différente — ce lien au brin a été délibérément écarté : aucun manuel ne justifie qu'un
brin précis mérite une meilleure note qu'un autre. Appliquer la même fenêtre de temps à tous les
avions du porte-avions (y compris le T-45, qui vole une pente différente) est une simplification
assumée, faute d'une référence de vitesse d'approche propre à chaque type.

## Étape 4 — Les compléments qui affinent `OK` en `(OK)` (jamais l'inverse)

Trois vérifications supplémentaires ne s'appliquent **que si** l'approche mériterait déjà un `OK`
d'après la grille ci-dessus. Elles ne peuvent jamais faire monter une note, seulement la retenir à
`(OK)` au lieu de `OK` :

- **La tendance des 4 dernières secondes.** Une vraie bonne approche, ce n'est pas seulement "être
  resté dans les clous" : c'est aussi "avoir bien corrigé". Deux pilotes qui finissent avec le même
  écart maximal ne se valent pas si l'un est arrivé haut puis a corrigé, et l'autre est arrivé
  propre puis a dérivé. Le programme regarde donc si l'écart, sur les 4 dernières secondes avant le
  toucher, est en train de **s'aggraver** plutôt que de se corriger. Si oui, la note plafonne à
  `(OK)`.
- **Les oscillations (survirage).** Un pilote qui corrige sans arrêt d'un côté puis de l'autre
  (trop à droite, correction, trop à gauche, correction...) peut, en moyenne sur 4 secondes,
  paraître stable — alors qu'il "pompe" en réalité. Le programme compte les changements de
  direction significatifs sur cette même fenêtre de 4 secondes ; à partir de deux changements de
  direction, la note plafonne aussi à `(OK)`, même si la tendance nette semblait bonne.
- **La zone des 150 derniers mètres avant le pont.** Le même écart n'a pas la même gravité selon
  qu'il se produit loin du pont (il reste du temps pour corriger) ou juste avant de toucher (il
  n'en reste plus). Si un écart modéré — plus marqué que la tolérance normale mais pas encore digne
  d'un `Cut` — se produit dans les 150 derniers mètres, la note est plafonnée à `--`, même si le
  reste de l'approche méritait `OK` ou `(OK)`.

Ces trois vérifications ne s'appliquent jamais à une approche déjà notée `--`, `C`, `B`, `WO?` ou
`NC` : elles ne servent qu'à distinguer un `OK` d'un `(OK)`, ou un `(OK)` d'un `--` pour la
pondération de fin d'approche.

## Ce qui n'est jamais noté (à une exception près)

Le programme enregistre et affiche, à titre purement informatif, plusieurs informations
supplémentaires qui n'influencent **jamais** la note, quelle que soit leur valeur :

- l'**AoA** (angle d'attaque), corrigé du vent quand c'est possible ;
- le **vent** (direction, vitesse) au moment de l'approche ;
- la **puissance moteur**, le **mouvement du pont**, et l'auteur réel d'une remise de gaz (jamais
  déduit, seulement quand il est prouvé par le jeu lui-même).

Ces valeurs servent uniquement au débrief (graphiques, contexte), jamais au calcul du score. La
raison : le projet n'a validé aucune règle fiable pour transformer, par exemple, "20 nœuds de vent
de travers" en points de note — inventer une telle règle serait moins honnête que de ne pas noter
du tout.

**Le taux de descente (sink rate) et l'inclinaison (bank/gîte) font exception, depuis le 5
septembre 2026** : sur leur amplitude générale, tout le long de l'approche, ils restent
informatifs comme le vent et l'AoA — pas de plafond `(OK)` comme pour la tendance ou les
oscillations (étape 4). Mais s'ils deviennent franchement dangereux et soutenus tout près du pont,
ils déclenchent directement un `C` (Cut) — voir l'étape 3 ci-dessus pour le détail et l'absence
volontairement assumée de base chiffrée officielle sur ce point précis.

## Résumé en une image

```
1. Trajectoire continue (groove → toucher)           → pire écart pente/alignement mesuré partout
2. Trois portes précises (3/4, 1/2, 1/4 NM)           → décident si l'appontage est notable du tout ;
                                                         combinées au continu (le pire des deux gagne)
3. Grille de base                                     → OK / (OK) / -- / C / B / WO? / NC
3bis. Fenêtre encore plus stricte + groove 15-18 s    → si déjà OK, peut monter à _OK_ (nouveau,
      (nouveau, 5 septembre 2026)                       5 septembre 2026) ; jamais pour un touch-and-go
4. Tendance 4 s, oscillations 4 s, zone des 150 m     → peut seulement faire descendre OK vers (OK)
                                                         ou (OK)/OK vers --, jamais remonter
5. Sink rate / gîte franchement dangereux, soutenu,   → fait directement tomber à C, comme le Cut
   près du pont (nouveau, 5 septembre 2026)             de pente — seul point sans base NATOPS chiffrée
6. AoA, vent, sink rate/gîte en dehors du cas ci-dessus, → affichés, jamais notés
   puissance
```

## Pourquoi "PROJECT-DERIVED" partout

Les seuils numériques exacts (0,5°, 1,0°, 2,5°, 150 m, 4 secondes, deux oscillations...) ne
viennent d'aucun manuel NAVAIR : ce sont des choix du projet, documentés et assumés comme tels,
construits pour être cohérents entre eux et défendables, mais **pas** une reconstitution certifiée
de la grille LSO réelle. C'est une note d'entraînement utile pour progresser, pas un substitut à un
vrai LSO humain.
