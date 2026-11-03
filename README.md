# Chop! (Horizon native mod)

**English** · [Français](README.fr.md)

**Cut one log with an axe: the whole tree tips over, crashes to the ground and breaks apart.**
A native Rust mod for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3).

## What it does

- **The log shakes while you cut it**: while you mine the log, it trembles and swells more and more,
  with little knocks, wood chips and a few leaves falling from the crown. When it gives way it swells
  one last time and pops in a small cloud. An axe that cuts the log in one hit pops it at once.
- **The fall**: the trunk first drops into the gap the log left and lands on the stump. The fibres
  crack, the trunk creaks, then gravity tips it over, like a rigid body pivoting on the front edge of
  its base. It starts slowly, speeds up, and stops where its **trunk** meets the ground or an obstacle:
  a dull wooden thud, snapping branches, the crown crushing, then a few pops. It bounces a little
  before it settles. A tall tree takes longer to fall than a small one.
- **Crushed leaves**: leaves no longer hold the tree up. Leaves caught between the trunk and the
  ground (or a wall) are crushed as it falls, with their particles and their loot.
- **A cute break-up**: once the tree has lain down (2.5 s by default), it disappears **from the base
  to the top**. Each block swells, hops and pops in a small cloud; the bubble pops rise in pitch as it
  goes, with small green sparkles; the last block leaves with a final pop and a few hearts.
- **Replanting** (setting): once the tree is gone, a sapling of its kind is planted where it stood.
  By default the sapling is **taken from the tree's loot** (no sapling dropped, nothing replanted); the
  "always" setting gives it for free. Every tree gives back its own sapling (oak, birch, spruce, jungle,
  acacia, dark oak, cherry, pale oak, mangrove propagule, azalea). A 2×2 trunk is replanted on its 4
  spots when there are enough saplings. It needs soil underneath; a tree cut above a stump is not
  replanted.
- **Chain reaction** (setting): if the falling trunk hits another tree's trunk, that tree tips over
  in turn, in the direction of the push. The first tree then finishes its fall, and so on, up to 3
  trees in a row by default.
- **In front of you**: the tree falls the way you look. Within an adjustable angle (±30° by default)
  it picks the clearest side; at 0° it falls exactly where you look.
- **Soft sounds, no resource pack**: like data packs do, Chop! only uses the game's own sounds and
  particles, layered and pitched, with no instruments: creaking and cracking wood when the tree starts
  to fall, a dull wooden thud softened with wool on impact, rustling and crushed leaves, bubble pops
  for the pops, wing flaps when the crown gains speed. Nothing else to install.
- **Dust and leaves**: on impact the ground that was hit (dirt, sand, snow…) puffs up, and leaves
  twirl while the tree lies on the ground.
- **Adjustable damage**: the tree hurts and knocks back what it hits, according to the speed of the
  block that hits. By default it only hurts **monsters**. Animals, allies (tamed animals, golems,
  villagers, allays) and players are spared; each group can be turned on on its own.
- **Real trees only**: the trunk must stand on something (cutting a branch does nothing); it needs some
  natural leaves; a neighbouring tree's leaves stay with their tree (vanilla's `distance` property);
  logs placed by a player protect the whole build, chain reactions included.
- **What hangs falls**: vines, cocoa, hanging moss, propagules, snow on the crown, bee nests, lichen
  and resin come off when the tree is cut.
- **Vanilla loot**: every block is mined with the tool you hold, Fortune and Silk Touch included (trees
  knocked over by a chain reaction too). An option sends all loot to your feet, another into your
  inventory.
- **Honest wear**: the axe takes one point of wear per log of the tree, Unbreaking included. An axe too
  worn for the tree only cuts one log, and the mod tells you.
- **Fall preview**: while you cut, a line of small golden particles on the ground shows where the trunk
  will land.
- **Shake a tree**: right-click a trunk with an empty hand to make the log tremble and leaves fall.
  Sometimes sticks, a sapling or an apple come out, once per tree every 30 s.
- **Giant mushrooms and Nether trees**: huge red and brown mushrooms and huge crimson and warped fungi
  fall like trees (stems, caps, wart blocks, shroomlights, vines), and are replanted as a mushroom or a
  fungus.
- **Angry bees**: a nest that falls without Silk Touch releases its bees, angry, as in vanilla.
- **Birds and apples**: when a big tree starts to tip, birds fly off the crown, chirping. Apples drop
  during the fall.
- **Biome colours**: twirling leaves take the biome's tint (oak, jungle, acacia, dark oak).
- **Lumberjack fatigue** (off by default): felling a tree makes you a little hungry.
- **Personal settings**: each player picks their own volume, particles and preview with `/chop perso`.
- **Stats**: `/chop stats` shows your felled trees, your record and the world's ranking.
- **World settings in game**, with commands or by clicking in the chat (see below).

## What it does not do

For players and reviewers, in plain words:

- **No network access**, no downloads, no telemetry. It never runs a program.
- **One file**: it reads and writes only `<world>/data/chop.txt` (changed settings, personal settings,
  stats, players who turned Chop! off, positions of logs placed by players). Nothing else on disk.
- **Only vanilla sounds and particles**, sent through the game's own queues. No resource pack.
- **Server-side**: it runs where the world runs (single player's server thread, LAN host, Horizon
  dedicated server). Players who join, Horizon or vanilla, install nothing.
- To act on the world it calls the game's own functions, and runs a few **vanilla commands** as the
  server (never as a player):
  - `kill @e[type=minecraft:block_display,tag=chop.…]`: removes its own falling-tree displays (only
    entities tagged by Chop!);
  - `item modify entity <player> weapon.mainhand …`: applies the axe's wear for the whole tree;
  - `damage <player> … minecraft:falling_block`: hurts a player the tree hits (off by default);
  - `summon minecraft:item` and `setblock … <sapling> keep`: drops loot and replants a sapling when the
    tree is in another dimension than the one being ticked.
- It only touches trees (logs, leaves, giant mushrooms, Nether fungi and what hangs on them), the
  sapling it replants, its own display entities, and the entities a falling tree hits.

## Install

The easy way: in the Horizon launcher, **Mods** page, search for **Chop!**, install it, then **Build**
("I trust these mods: build"). The launcher compiles it on the Nightly it was made for, on its own.

By hand, with the `.hnmod` file:

You need the **Horizon launcher** (Linux, macOS Apple Silicon or Windows). On Linux, building native mods
needs `sudo apt install build-essential pkg-config libasound2-dev libudev-dev` (Debian/Ubuntu/Mint).

1. In the launcher, **Mods** page, pick the instance and **Open folder** (or open the instance's
   `horizon-native-mods/` folder yourself, next to its `options.txt`).
2. Copy `chop-1.0.0.hnmod` into `horizon-native-mods/` (and remove any older version).
3. Turn it on with its switch on the Mods page, or add the line `mod.chop:true` to `horizon-mods.txt`.
4. On the **Native build** row, click **Build**, then "I trust these mods: build". The first build
   takes 20 to 30 minutes.
5. Start the instance and check in **Options > Mods** that Chop! is on.

If you have another tree-felling mod, turn it off: both mods would fell the same tree.

From the command line, in the unpacked source folder (the one with `horizon-mixin.toml`):

```sh
horizon-launcher native-build /path/to/chop/mixins/chop --instance "Horizon" --trust-native-code
```

Chop! is **pinned to Horizon Nightly 2026-11-01** (`c9d474d`): it uses some of the game's internal
functions (a block's loot, dropped items, mobs, the sound and particle queues), so the launcher always
builds it on that commit, even if you play a newer Nightly. New versions of Chop! will follow the
Nightlies.

On a Horizon **dedicated server**: `horizon-launcher native-build <folder> --server-output
<server-folder>`. Players (Horizon or vanilla) install nothing.

## Use

Hold an axe and cut a log at the bottom of a tree. Sneaking, you only cut one log (adjustable).

The commands and settings are in French, the mod's language.

| Command | Who | What |
|---|---|---|
| `/chop` | everyone | Turns Chop! on or off for yourself (`/chop on`, `/chop off`) |
| `/chop aide` | everyone | The commands |
| `/chop perso` | everyone | Your own settings (sounds, particles, preview), clickable |
| `/chop stats` | everyone | Your felled trees and the ranking |
| `/chop config` | operators and the single-player world's owner | The clickable settings menu |
| `/chop config chute\|effets\|degats\|regles` | same | One page of the menu |
| `/chop set <setting> <value>` | same | Changes a setting |
| `/chop reset` | same | Restores the default settings |

In the menu each page has its tabs **[Chute] [Effets] [Dégâts] [Règles]** (fall, effects, damage,
rules). Yes/no settings have two buttons **[oui] [non]**, numbers have **[-]** and **[+]**. Hover a
name to read what it does. In single player you can open the menu without cheats.

## Settings

| Page | Setting | Default | Values |
|---|---|---|---|
| Chute | `animation` | yes | no: the tree breaks on the spot, at once |
| Chute | `vitesse` | 200 % | 50 to 400 % (gravity of the fall; 100 % is realistic) |
| Chute | `devant` | yes | no: the tree falls on its clearest side |
| Chute | `angle` | ±30° | 0 to 90°: how far from your view it may fall |
| Chute | `pose` | 2.5 s | 0 to 10 s on the ground before it breaks up (in tenths: `pose 25`) |
| Chute | `chaine` | 3 trees | 0 (off) to 10: trees knocked over in a row by a chain reaction |
| Effets | `sons` | 100 % | 0 to 200 % |
| Effets | `particules` | normal | none, few, normal, many (0 to 3) |
| Effets | `feuilles` | yes | twirling leaves |
| Effets | `poussiere` | yes | ground dust on impact |
| Effets | `secousse` | yes | the log shakes and swells while you cut it, then pops |
| Effets | `apercu` | yes | line of particles showing where the tree will fall |
| Effets | `abeilles` | yes | a falling nest releases its bees, angry |
| Effets | `oiseaux` | yes | birds fly off big trees |
| Effets | `pommes` | yes | apples drop during the fall |
| Dégâts | `degats` | yes | the tree hurts what it hits |
| Dégâts | `force` | 5 ❤ | 1 to 40 health points at most per hit |
| Dégâts | `monstres` | yes | |
| Dégâts | `animaux` | no | passive and neutral creatures without an owner |
| Dégâts | `allies` | no | tamed animals, golems, villagers, allays |
| Dégâts | `joueurs` | no | you included |
| Règles | `actif` | yes | no: Chop! does nothing in this world |
| Règles | `hache` | yes | an axe is required |
| Règles | `accroupi` | 0 | 0: sneaking = one log; 1: sneak to fell; 2: always fell |
| Règles | `usure` | yes | the axe wears for the whole tree |
| Règles | `butin` | 0 | 0: where it falls; 1: at the player's feet; 2: into the inventory |
| Règles | `max_buches` | 256 | above it, the tree is cut log by log |
| Règles | `max_blocs` | 1200 | logs and leaves included |
| Règles | `rayon` | 12 | farthest a branch may be from the cut log |
| Règles | `protection` | yes | logs placed by a player never fall |
| Règles | `replanter` | 1 | 0: no; 1: a sapling from the loot; 2: always (free) |
| Règles | `secouer` | yes | right-click with an empty hand shakes the tree |
| Règles | `fatigue` | no | felling a tree makes you a little hungry |
| Règles | `champignons` | yes | giant mushrooms and Nether fungi fall too |

In creative mode the tree falls too, with no loot and no wear.

Nether fungi never drop their own fungus: with "a sapling from the loot" they are replanted anyway.

Personal settings (`/chop perso`, or `/chop perso <setting> <value>`): `sons` (0 to 200 %),
`particules` (0 to 3), `apercu` (0 or 1). The value -1, or the [comme le monde] button, follows the
world's setting.

## What was checked

**In the real game.** Horizon was run without a window, with software rendering, on Nightly `c9d474d`
with the real Minecraft 26.3 data. A player was driven through Horizon's agent control: it mines logs as
a player does, and screenshots were taken. Seen and measured:

- An oak in survival: the log shakes while being cut; the tree tips toward where the player looks,
  lies down, breaks up from the base to the top with clouds, sparkles and hearts; every display is
  removed afterwards; the loot drops (logs, sticks, saplings, apple) and a sapling is replanted.
- In a real generated forest: felled and replanted, with no error in the game's log.
- Loot "into the inventory": logs, saplings, sticks and apple reach the inventory.
- A huge crimson fungus on nylium falls with its wart blocks, shroomlight and vines, then is replanted.
- A huge red mushroom on mycelium falls with its whole cap, then is replanted.
- Chain reaction: a cut oak knocks over the oak next to it, and both break up.
- A bee nest on the trunk: its 2 bees are released.
- Damage: a zombie in the way loses health, a cow next to it does not.
- Shaking a tree with an empty hand: the log trembles and leaves fall.
- `/chop stats`, `/chop perso` and the `/chop config` menu work in the chat, buttons included.
- The particles of the preview, birds, sparkles, hearts and impact dust reach the player.

**Outside the game.** The rules (`mixins/chop/core.rs`) pass 18 tests on hand-built worlds. The bundle
applies with the launcher's horizon-mixins, the modded game builds completely with the launcher's `fast`
profile, and the `.hnmod` passes `cargo horizon-mod check`.

**Not checked**: the sound output itself (the test game had no audio device; only the sending was
checked), the fatigue's size, and play with several players.

## Known limits

- During the fall, the tree's leaves have the game's default colour, not the biome's: the game draws
  display blocks that way.
- The whole falling tree is lit like the cut block.
- Shaking a tree doesn't give much: on purpose, so it isn't a farm.

## Build

```sh
cargo test                     # the rules, without Horizon
python3 tools/package.py       # target/chop-1.0.0.hnmod (reproducible: same sources, same SHA-256)
```

## Licence

© 2026 Mayonesu0, all rights reserved, except what the [licence](LICENSE) allows. The source is public
so that players and Horizon's reviewers can see what it does, but it is not free software:

- you may download Chop!, have the launcher build it, and play with it, unchanged;
- Horizon's registry may distribute it, and its reviewers read it to review it;
- you may not copy its code, modify it, or redistribute a modified version without Mayonesu0's written
  permission.

To ask for a permission, open an issue on this repository.

Chop! is not an official Minecraft product, and is not approved by or associated with Mojang or
Microsoft. Horizon is a separate project, under its own licence.
