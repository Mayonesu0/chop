# Chop! (mod natif Horizon)

[English](README.md) · **Français**

**Coupe une bûche à la hache : tout l'arbre bascule, s'abat au sol et se brise.**
Mod natif en Rust pour [Horizon](https://github.com/horizon-rust/horizon) (Minecraft Java 26.3),
écrit uniquement sur l'API de mods d'Horizon.

## Ce que fait le mod

- **La bûche qui tremble pendant qu'on la coupe** : à mesure que la fissure grandit dans la bûche, elle
  tremble et gonfle de plus en plus fort, avec des petits « toc », des copeaux et quelques feuilles qui
  tombent de la couronne. Quand elle cède, elle gonfle une dernière fois et éclate dans un petit nuage
  avec un « plop » bubbly. Une hache qui coupe la bûche presque d'un coup la fait éclater aussitôt.
- **La chute** : le tronc tombe d'abord dans le trou laissé par la bûche et atterrit sur la souche
  (« pomf »). Les fibres craquent, le tronc grince, puis il bascule sous l'effet de la gravité, comme un
  corps rigide qui pivote sur le
  bord avant de sa base. Il part lentement, accélère, puis s'arrête là où son **tronc** touche le sol
  ou un obstacle : un choc de bois sourd, des branches qui cassent, la couronne qui s'écrase, puis
  quelques pops. Il rebondit un peu (« boing ») avant de se poser. Un grand arbre met plus de temps
  qu'un petit.
- **Feuilles écrasées** : les feuilles ne retiennent plus l'arbre. Celles qui se retrouvent prises entre
  le tronc et le sol (ou un mur) sont écrasées en tombant, avec leurs particules et leur butin.
- **Une désagrégation mignonne** : une fois couché (2,5 s par défaut), l'arbre disparaît **de la base
  vers la cime**.
  - Chaque bloc gonfle, saute un peu et éclate en petit nuage.
  - Les pops de bulles montent dans les aigus au fil de la désagrégation, avec de petites étincelles
    vertes.
  - Le dernier bloc parti, un dernier pop et quelques cœurs.
- **Replantation** (réglable) : une fois l'arbre parti, une pousse de son essence est replantée là où
  il se tenait, avec un petit pop et des étincelles vertes.
  - Par défaut, la pousse est **prise dans le butin** de l'arbre : s'il n'en a pas donné, rien n'est
    replanté. Le réglage « toujours » l'offre.
  - Chaque arbre redonne sa propre pousse : chêne, bouleau, sapin, acajou, acacia, chêne noir,
    cerisier, chêne pâle, propagule de palétuvier. Un azalée redonne un azalée (fleuri ou non).
  - Un tronc 2×2 (chêne noir, grand sapin, grand acajou…) est replanté à ses 4 places, s'il y a assez
    de pousses.
  - Il faut de la terre dessous (terre, herbe, podzol, mousse, boue…). Si tu as coupé au-dessus d'une
    souche, rien n'est replanté ; si la place a été prise entre-temps, la pousse tombe en objet.
- **Réaction en chaîne** (réglable) : si le tronc qui tombe percute le tronc d'un autre arbre, celui-ci
  bascule à son tour dans le sens de la poussée (« bonk »). L'arbre qui l'a percuté continue alors sa
  chute, et ainsi de suite, jusqu'à 3 arbres d'affilée par défaut.
- **Devant toi** : l'arbre tombe dans la direction où tu regardes. Dans un angle réglable (±30° par
  défaut), il prend le côté le plus dégagé. À 0°, il tombe exactement où tu regardes.
- **Des sons doux, sans pack de ressources** : comme les data packs, Chop! n'utilise que les sons et
  particules du jeu, superposés et accordés, sans instrument :
  - le bois qui craque et grince quand l'arbre commence à tomber ;
  - un choc de bois sourd, adouci par la laine, à l'impact ;
  - le feuillage qui froisse en tombant et s'écrase au sol ;
  - des pops de bulles et des « plops » pour les éclatements ;
  - un battement d'ailes quand la couronne prend de la vitesse.
  Il n'y a rien à installer à côté.
- **Poussière et feuilles** : à l'impact, le sol touché (terre, sable, neige…) se soulève, et des
  feuilles virevoltent tant que l'arbre est couché.
- **Dégâts réglables** : l'arbre blesse et repousse ce qu'il percute, selon la vitesse du bloc qui
  touche. Par défaut il ne touche que les **monstres**. Les animaux, les alliés (animaux apprivoisés,
  golems, villageois, allays) et les joueurs sont épargnés, et chaque catégorie s'active à part.
- **Un vrai arbre seulement** :
  - le tronc doit reposer sur quelque chose (couper une branche ne fait rien) ;
  - il faut au moins quelques feuilles naturelles ;
  - les feuilles d'un arbre voisin restent à leur arbre (propriété `distance` de vanilla) ;
  - les bûches posées par un joueur protègent la construction entière, y compris contre une réaction en
    chaîne.
- **Ce qui pend tombe** : lianes, cacao, mousse pendante, propagules, neige sur la couronne, nids
  d'abeilles, lichen et résine se détachent dès la coupe.
- **Butin vanilla** : chaque bloc est miné avec l'outil que tu tiens, Fortune et Toucher de soie compris
  (les arbres renversés par la chaîne aussi). Une option envoie tout le butin à tes pieds.
- **Usure honnête** : la hache s'use d'un point par bûche de l'arbre que tu coupes, Solidité comprise.
  Une hache trop usée pour l'arbre ne coupe qu'une bûche, et le mod te le dit.
- **Aperçu de la chute** : pendant que tu coupes, une ligne de petites particules dorées au sol montre où
  le tronc va se coucher.
- **Secouer un arbre** : un clic droit à main nue sur le tronc fait trembler la bûche et tomber des
  feuilles. Il en sort parfois des bâtons, des pousses ou une pomme, une fois par arbre toutes les 30 s.
- **Champignons géants et arbres du Nether** : les champignons géants rouges et bruns, et les champignons
  géants carmin et biscornus, tombent comme des arbres (tiges, chapeaux, blocs de verrue, champilumières,
  lianes pleureuses). Ils sont replantés en champignon ou en champignon du Nether.
- **Abeilles en colère** : un nid qui tombe sans Toucher de soie relâche ses abeilles, en colère : il est
  cassé comme le bûcheron le casserait, selon les règles mêmes de vanilla.
- **Oiseaux et pommes** : quand un grand arbre commence à basculer, des oiseaux s'envolent de la
  couronne en pépiant. Les pommes se détachent pendant la chute.
- **Couleurs du biome** : les feuilles qui virevoltent prennent la teinte du biome (chêne, acajou, acacia,
  chêne noir) dans les biomes où poussent ces arbres (voir [Limites connues](#limites-connues)).
- **Fatigue du bûcheron** (désactivée par défaut) : abattre un arbre creuse un peu la faim.
- **Butin dans l'inventaire** (option) : tout va directement dans ton inventaire, le surplus à tes pieds.
- **Réglages perso** : chacun choisit son volume, ses particules et l'aperçu avec `/chop perso`.
- **Statistiques** : `/chop stats` montre tes arbres abattus, ton record et le classement du monde.
- **Réglages du monde en jeu**, par commandes ou en cliquant dans le chat (voir plus bas).

## Ce que le mod ne fait pas

Pour les joueurs et les relecteurs, en clair :

- **Aucun accès réseau**, aucun téléchargement, aucune télémétrie. Il ne lance aucun programme.
- **Un seul fichier** : il ne lit et n'écrit que `<monde>/data/chop.txt` (réglages modifiés, réglages
  perso, statistiques, joueurs qui ont désactivé Chop!, positions des bûches posées). Rien d'autre sur le
  disque.
- **Uniquement des sons et particules vanilla**, envoyés par l'API de mods. Aucun pack de ressources.
- **Côté serveur** (`side = "server"`) : il tourne là où tourne le monde (fil serveur du solo, hôte LAN,
  serveur dédié Horizon). Les joueurs qui rejoignent, Horizon ou vanilla, n'installent rien.
- **Uniquement l'API de mods d'Horizon** : il ne modifie rien dans le jeu (aucun mixin, aucun accès
  interne). Pour agir sur le monde, il appelle les fonctions de l'API (blocs, butin, objets, entités,
  blocs affichés, sons, particules), et lance quelques **commandes vanilla** en tant que serveur (jamais
  en tant que joueur), chacune dans la dimension de l'arbre (`execute in <dimension> run …`) :
  - `kill @e[type=minecraft:block_display,tag=chop.…]` : retire ses propres blocs affichés (seulement les
    entités marquées par Chop!) ;
  - `item modify entity <joueur> weapon.mainhand …` : applique l'usure de la hache pour tout l'arbre ;
  - `damage <joueur> … minecraft:falling_block` : blesse un joueur touché par l'arbre (désactivé par
    défaut) ;
  - `playsound … <joueur>` et `particle … normal <joueur>` : un son ou des particules pour un seul joueur,
    quand les joueurs proches n'ont pas tous les mêmes réglages perso (sinon, c'est un seul son, ou un
    seul envoi, pour tous ceux qui sont près) ;
  - `summon minecraft:item` et `setblock … <pousse> keep` : fait tomber le butin et replante une pousse
    quand un arbre doit finir alors que sa dimension ne tourne pas (tout le monde l'a quittée, ou le
    serveur s'arrête).
- Il ne touche qu'aux arbres (bûches, feuilles, champignons géants, champignons du Nether et ce qui y
  pend), à la pousse qu'il replante, à ses propres entités affichées et aux entités que l'arbre percute.

## Installer

Le plus simple : dans le launcher Horizon, page **Mods**, cherche **Chop!**, installe-le, puis
**Build** (« I trust these mods: build »).

Directement depuis ce dépôt (pour une version pas encore dans le registre) : sur la page Mods, **Add from
GitHub** avec `https://github.com/Mayonesu0/chop`, ou en ligne de commande :

```sh
horizon-launcher mods add-github https://github.com/Mayonesu0/chop --trust-native-code
```

À la main, avec le fichier `.hmod` :

Il faut le **launcher Horizon** (Linux, macOS Apple Silicon ou Windows). Sous Linux, la compilation des
mods natifs peut demander : `sudo apt install build-essential pkg-config libasound2-dev libudev-dev`
(Debian/Ubuntu/Mint).

1. Fabrique le paquet dans ce dossier avec l'outil de mods d'Horizon : `cargo horizon-mod build` écrit
   `target/horizon-mod/chop.hmod` (l'outil : `cargo install --locked --git
   https://github.com/horizon-rust/horizon cargo-horizon-mod`).
2. Dans le launcher, page **Mods**, choisis l'instance puis **Open folder** (ou ouvre toi-même le dossier
   `horizon-native-mods/` du dossier de jeu de l'instance, celui qui contient `options.txt`).
3. Copie `chop.hmod` dans `horizon-native-mods/`, et retire tout ancien `chop-*.hnmod` : le launcher
   refuse l'ancien format.
4. Active-le avec son interrupteur sur la page Mods, ou ajoute la ligne `mod.chop:true` dans
   `horizon-mods.txt`.
5. Sur la ligne **Native build**, clique **Build**, puis « I trust these mods: build ». Le jeu est déjà
   compilé : la première fois, le launcher télécharge le SDK de la build (quelques centaines de Mo) et
   compile Chop! dessus.
6. Lance l'instance et vérifie dans **Options > Mods** que Chop! est activé.

Si tu as un autre mod d'abattage d'arbres, désactive-le : les deux mods abattraient le même arbre.

En ligne de commande, dans ce dossier (celui qui contient `horizon-mod.toml`) :

```sh
horizon-launcher native-build . --instance "Horizon" --trust-native-code
```

**Quelle Nightly.** Chop! 1.1.0 est fait pour la **Nightly Horizon 2026-11-22** (`f1bb66f`). Il n'utilise
que l'API de mods, et son paquet liste ce qu'il en utilise (`uses.json`) : le launcher le compile donc
aussi sur une Nightly plus récente où rien de tout ça n'a changé (`cargo horizon-mod fits` te le dit).
Quand une Nightly change quelque chose dont Chop! se sert, le launcher le laisse de côté pour cette build
en nommant ce qui a changé, jusqu'à ce qu'une nouvelle version suive.

Sur un **serveur dédié** Horizon : `horizon-launcher native-build <dossier> --server-output
<dossier-serveur> --version <tag> --trust-native-code`. Les joueurs (Horizon ou vanilla) n'ont rien à
installer.

## Utilisation

Tiens une hache et coupe une bûche du bas d'un arbre. Accroupi, tu ne coupes qu'une bûche (réglable).

| Commande | Qui | Effet |
|---|---|---|
| `/chop` | tout le monde | Active ou désactive Chop! pour soi (`/chop on`, `/chop off`) |
| `/chop aide` | tout le monde | Les commandes |
| `/chop perso` | tout le monde | Tes propres réglages (sons, particules, aperçu), cliquables |
| `/chop stats` | tout le monde | Tes arbres abattus et le classement |
| `/chop config` | opérateurs et propriétaire du monde solo | Le menu de réglages cliquable |
| `/chop config chute\|effets\|degats\|regles` | idem | Une page du menu |
| `/chop set <réglage> <valeur>` | idem | Change un réglage |
| `/chop reset` | idem | Remet les réglages par défaut |

Dans le menu, chaque page a ses onglets **[Chute] [Effets] [Dégâts] [Règles]**. Les réglages
oui/non ont deux boutons **[oui] [non]**, les nombres **[-]** et **[+]**. Survole un nom pour lire son
explication. En solo, tu peux ouvrir le menu même sans les triches.

## Réglages

| Page | Réglage | Défaut | Valeurs |
|---|---|---|---|
| Chute | `animation` | oui | non : l'arbre se brise sur place, d'un coup |
| Chute | `vitesse` | 200 % | 50 à 400 % (gravité de la chute ; 100 % est réaliste) |
| Chute | `devant` | oui | non : l'arbre tombe du côté le plus dégagé |
| Chute | `angle` | ±30° | 0 à 90° : écart permis avec ton regard |
| Chute | `pose` | 2,5 s | 0 à 10 s couché avant de se désagréger (en dixièmes : `pose 25`) |
| Chute | `chaine` | 3 arbres | 0 (non) à 10 : arbres renversés d'affilée par une réaction en chaîne |
| Effets | `sons` | 100 % | 0 à 200 % |
| Effets | `particules` | normales | aucune, peu, normales, beaucoup (0 à 3) |
| Effets | `feuilles` | oui | feuilles qui virevoltent |
| Effets | `poussiere` | oui | poussière du sol à l'impact |
| Effets | `secousse` | oui | la bûche tremble et gonfle pendant que tu la coupes, puis éclate |
| Effets | `apercu` | oui | ligne de particules au sol montrant où l'arbre va tomber |
| Effets | `abeilles` | oui | un nid qui tombe relâche ses abeilles en colère |
| Effets | `oiseaux` | oui | des oiseaux s'envolent des grands arbres |
| Effets | `pommes` | oui | les pommes se détachent pendant la chute |
| Dégâts | `degats` | oui | l'arbre blesse ce qu'il percute |
| Dégâts | `force` | 5 ❤ | 1 à 40 points de vie au plus par coup |
| Dégâts | `monstres` | oui | |
| Dégâts | `animaux` | non | créatures passives et neutres sans maître |
| Dégâts | `allies` | non | apprivoisés, golems, villageois, allays |
| Dégâts | `joueurs` | non | toi compris |
| Règles | `actif` | oui | non : Chop! ne fait plus rien sur ce monde |
| Règles | `hache` | oui | une hache est obligatoire |
| Règles | `accroupi` | 0 | 0 : accroupi = une bûche ; 1 : accroupi pour abattre ; 2 : toujours abattre |
| Règles | `usure` | oui | la hache s'use pour tout l'arbre |
| Règles | `butin` | 0 | 0 : là où il tombe ; 1 : aux pieds du joueur ; 2 : dans l'inventaire |
| Règles | `max_buches` | 256 | au-delà, l'arbre se coupe bûche par bûche |
| Règles | `max_blocs` | 1200 | bûches et feuilles comprises |
| Règles | `rayon` | 12 | distance maximale d'une branche à la bûche coupée |
| Règles | `protection` | oui | les bûches posées par un joueur ne tombent jamais |
| Règles | `replanter` | 1 | 0 : non ; 1 : une pousse du butin ; 2 : toujours (offerte) |
| Règles | `secouer` | oui | un clic droit à main nue secoue l'arbre |
| Règles | `fatigue` | non | abattre un arbre creuse un peu la faim |
| Règles | `champignons` | oui | les champignons géants et les arbres du Nether tombent aussi |

En mode créatif, l'arbre tombe aussi, sans butin et sans usure.

Les champignons du Nether ne donnent jamais leur propre pousse : avec « une pousse du butin », ils sont
replantés quand même.

Réglages perso (`/chop perso`, ou `/chop perso <réglage> <valeur>`) : `sons` (0 à 200 %), `particules`
(0 à 3), `apercu` (0 ou 1). La valeur -1, ou le bouton [comme le monde], suit le réglage du monde.

Les données du monde (réglages modifiés, réglages perso, statistiques, joueurs qui ont désactivé Chop!,
bûches posées) sont dans `<monde>/data/chop.txt`.

## Ce qui a été vérifié

**La version 1.1.0** réécrit le côté jeu (`src/game.rs`, `src/lib.rs`) pour l'API de mods d'Horizon ; les
règles (`src/rules.rs`) n'ont pas changé.

- Les règles passent leurs 18 tests sur des mondes construits à la main
  (`cargo test --manifest-path rules/Cargo.toml`).
- Chaque hook, événement, méthode, type et import utilisé par le côté jeu a été comparé aux sources du
  SDK épinglé (la release signée `sdk-f1bb66f…`) : tout correspond. Il compile aussi, sans aucun
  avertissement de clippy, contre une petite copie de ces déclarations.
- **Pas encore vérifié** : une vraie compilation avec le SDK (`cargo horizon-mod check` ; les dépendances
  crates.io de l'API n'ont pas pu être téléchargées là où il a été écrit), et une partie dans le jeu.
  D'ici là, considère la 1.1.0 comme non testée.

**La version 1.0.1, dans le vrai jeu.** Horizon a été lancé sans fenêtre, avec un rendu logiciel, sur la
Nightly `c9d474d` avec les vraies données de Minecraft 26.3. Un joueur a été piloté par le mode agent
d'Horizon : il mine les bûches comme un joueur, et des captures d'écran ont été prises. Ce qui a été vu et
mesuré :

- Un chêne en survie :
  - la bûche tremble pendant la coupe ;
  - l'arbre bascule du côté où l'on regarde, se couche, se désagrège de la base vers la cime avec
    nuages, étincelles et cœurs ;
  - tous les blocs affichés disparaissent ensuite ;
  - le butin tombe (bûches, bâtons, pousses, pomme) et une pousse est replantée au pied.
- Dans une vraie forêt générée : abattu et replanté, sans erreur dans le journal du jeu.
- Butin « dans l'inventaire » : bûches, pousses, bâtons et pomme arrivent dans l'inventaire.
- Champignon géant carmin sur nylium : tombe avec ses blocs de verrue, sa champilumière et ses lianes,
  puis est replanté en champignon carmin.
- Champignon rouge géant sur mycélium : tombe avec tout son chapeau, puis est replanté en champignon
  rouge.
- Réaction en chaîne : un chêne coupé renverse le chêne voisin et les deux se désagrègent.
- Nid d'abeilles sur le tronc : ses 2 abeilles sont relâchées.
- Dégâts : un zombie sur la trajectoire perd des points de vie, une vache à côté n'en perd pas.
- Secouer un arbre à main nue : la bûche tremble et des feuilles tombent.
- `/chop stats`, `/chop perso` et le menu `/chop config` fonctionnent dans le chat, boutons compris.
- Les particules de l'aperçu, des oiseaux, des étincelles, des cœurs et de la poussière d'impact arrivent
  bien au joueur.

Deux défauts trouvés en jeu ont été corrigés :
- les côtés du chapeau du champignon rouge restaient suspendus ;
- le champignon rouge n'était pas replanté, car une étiquette de blocs a changé de nom en 26.3.

**La version 1.0.1, hors du jeu.** Les règles passaient les mêmes 18 tests ; le bundle s'appliquait avec
l'outil du launcher, le jeu modifié se compilait entièrement avec le profil `fast` du launcher, et le
`.hnmod` était accepté par le lecteur de bundles du launcher.

**Jamais vérifié** :
- le son : le jeu de test n'avait pas de sortie audio, seul l'envoi des sons a été vérifié ;
- la fatigue : trop faible pour se voir sur un petit arbre ;
- le jeu avec plusieurs joueurs.

## Limites connues

- Pendant la chute, les feuilles de l'arbre ont la couleur par défaut du jeu, pas celle du biome : le
  jeu dessine ainsi les blocs affichés.
- Les feuilles qui virevoltent prennent la teinte du biome dans les plaines, les forêts (forêts de fleurs
  et forêts noires comprises), les jungles, les savanes, les marais (marais de palétuviers compris), les
  badlands et sur les plages. Ailleurs, et dans les biomes ajoutés par des data packs ou des mods, elles
  gardent la couleur habituelle des feuilles : le serveur ne connaît pas la palette des biomes.
- Tout l'arbre qui tombe est éclairé comme le bloc coupé.
- La bûche commence à trembler à la première fissure : une bûche coupée en un ou deux coups ne tremble
  pas.
- Un nid d'abeilles qui tombe relâche ses abeilles quand le bûcheron est en survie ou en aventure, dans la
  dimension de l'arbre. En créatif, ses abeilles restent dedans, comme quand on le casse en créatif en
  vanilla.
- Secouer un arbre ne donne pas grand-chose : c'est voulu, pour que ce ne soit pas une ferme.

## Développement

```sh
cargo horizon-mod check                       # règles du paquet, cargo check avec le SDK épinglé, ce que Chop! utilise
cargo horizon-mod build                       # target/horizon-mod/chop.hmod, avec son uses.json
cargo horizon-mod fits                        # s'il convient à la Nightly actuelle
cargo test --manifest-path rules/Cargo.toml   # les règles, sans Horizon
```

La première commande `cargo horizon-mod` télécharge le SDK épinglé dans `.horizon/`, qui n'est ni dans le
dépôt ni dans le paquet.

| Chemin | Contenu |
|---|---|
| `horizon-mod.toml` | Le manifeste : identifiant, version, la build pour laquelle il est fait |
| `src/lib.rs` | Les hooks : chaque moment du jeu que Chop! écoute |
| `src/game.rs` | Le côté jeu : lit et modifie le monde par l'API de mods |
| `src/rules.rs` | Les règles : l'arbre, sa chute, la physique, les réglages (sans Horizon) |
| `rules/` | Une petite crate qui teste les règles seules (hors du paquet) |
| `icon.png` | L'icône que montrent le launcher et l'écran Mods du jeu |
| `registry/` | La page du registre et son icône |

## Licence

© 2026 Mayonesu0, tous droits réservés, sauf ce que la [licence](LICENSE) permet. Le code est public pour
que les joueurs et les relecteurs d'Horizon puissent voir ce qu'il fait, mais ce n'est pas un logiciel
libre :

- tu peux télécharger Chop!, le faire compiler par le launcher et jouer avec, sans le modifier ;
- le registre d'Horizon peut le distribuer, et ses relecteurs le lire pour le relire ;
- tu ne peux pas copier son code, le modifier ou redistribuer une version modifiée sans l'autorisation
  écrite de Mayonesu0.

Pour demander une autorisation, ouvre une issue sur ce dépôt.

Chop! n'est pas un produit officiel Minecraft, et n'est ni approuvé par Mojang ou Microsoft, ni associé
à eux. Horizon est un projet séparé, sous sa propre licence.
