# Chop! (Horizon native mod)

**English** · [Français](README.fr.md)

**Cut one log with an axe: the whole tree tips over, crashes to the ground and breaks apart.**
A native Rust mod for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft: Java Edition 26.3),
written on Horizon's mod API alone.

## What it does

- **The log shakes while you cut it**: as the crack in the log grows, it trembles and swells more and
  more, with little knocks, wood chips and a few leaves falling from the crown. When it gives way it
  swells one last time and pops in a small cloud. An axe that cuts the log almost at once pops it
  straight away.
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
- **Angry bees**: a nest that falls without Silk Touch releases its bees, angry: it is broken as the
  lumberjack breaks it, by vanilla's own rules.
- **Birds and apples**: when a big tree starts to tip, birds fly off the crown, chirping. Apples drop
  during the fall.
- **Biome colours**: twirling leaves take the biome's tint (oak, jungle, acacia, dark oak) in the biomes
  where those trees grow (see [Known limits](#known-limits)).
- **Lumberjack fatigue** (off by default): felling a tree makes you a little hungry.
- **Personal settings**: each player picks their own volume, particles and preview with `/chop perso`.
- **Stats**: `/chop stats` shows your felled trees, your record and the world's ranking.
- **World settings in game**, with commands or by clicking in the chat (see below).

## What it does not do

For players and reviewers, in plain words:

- **No network access**, no downloads, no telemetry. It never runs a program.
- **One file**: it reads and writes only `<world>/data/chop.txt` (changed settings, personal settings,
  stats, players who turned Chop! off, positions of logs placed by players). Nothing else on disk.
- **Only vanilla sounds and particles**, sent through the mod API. No resource pack.
- **Server-side** (`side = "server"`): it runs where the world runs (single player's server thread, LAN
  host, Horizon dedicated server). Players who join, Horizon or vanilla, install nothing.
- **Horizon's mod API only**: it patches nothing in the game (no mixin, no internals). To act on the
  world it calls the API's functions (blocks, loot, items, entities, displays, sounds, particles), and
  runs a few **vanilla commands** as the server (never as a player), each in the tree's dimension
  (`execute in <dimension> run …`):
  - `kill @e[type=minecraft:block_display,tag=chop.…]`: removes its own falling-tree displays (only
    entities tagged by Chop!);
  - `item modify entity <player> weapon.mainhand …`: applies the axe's wear for the whole tree;
  - `damage <player> … minecraft:falling_block`: hurts a player the tree hits (off by default);
  - `playsound … <player>` and `particle … normal <player>`: a sound or particles for one player, when
    the players nearby don't all have the same personal settings (otherwise it is one sound, or one
    send, for everyone near it);
  - `summon minecraft:item` and `setblock … <sapling> keep`: drops loot and replants a sapling when a
    tree has to end while its dimension isn't ticking (everyone left it, or the server stops).
- It only touches trees (logs, leaves, giant mushrooms, Nether fungi and what hangs on them), the
  sapling it replants, its own display entities, and the entities a falling tree hits.

## Install

The easy way: in the Horizon launcher, **Mods** page, search for **Chop!**, install it, then **Build**
("I trust these mods: build").

Straight from this repository (for a version not yet in the registry): on the Mods page, **Add from
GitHub** with `https://github.com/Mayonesu0/chop`, or from the command line:

```sh
horizon-launcher mods add-github https://github.com/Mayonesu0/chop --trust-native-code
```

By hand, with the `.hmod` file:

You need the **Horizon launcher** (Linux, macOS Apple Silicon or Windows). On Linux, building native mods
may need `sudo apt install build-essential pkg-config libasound2-dev libudev-dev` (Debian/Ubuntu/Mint).

1. Make the package in this folder with Horizon's mod tool: `cargo horizon-mod build` writes
   `target/horizon-mod/chop.hmod` (the tool: `cargo install --locked --git
   https://github.com/horizon-rust/horizon cargo-horizon-mod`).
2. In the launcher, **Mods** page, pick the instance and **Open folder** (or open the instance's
   `horizon-native-mods/` folder yourself, next to its `options.txt`).
3. Copy `chop.hmod` into `horizon-native-mods/`, and remove any older `chop-*.hnmod`: the launcher refuses
   the old format.
4. Turn it on with its switch on the Mods page, or add the line `mod.chop:true` to `horizon-mods.txt`.
5. On the **Native build** row, click **Build**, then "I trust these mods: build". The game is prebuilt:
   the launcher fetches the build's SDK the first time (a few hundred MB) and compiles Chop! against it.
6. Start the instance and check in **Options > Mods** that Chop! is on.

If you have another tree-felling mod, turn it off: both mods would fell the same tree.

From the command line, in this folder (the one with `horizon-mod.toml`):

```sh
horizon-launcher native-build . --instance "Horizon" --trust-native-code
```

**Which Nightly.** Chop! 1.1.0 is made for **Horizon Nightly 2026-11-22** (`f1bb66f`). It uses only the
mod API, and its package lists what it uses (`uses.json`), so the launcher also builds it on a later
Nightly where none of that changed (`cargo horizon-mod fits` tells you). When a Nightly changes something
Chop! uses, the launcher leaves it out of that build and names what changed, until a new version follows.

On a Horizon **dedicated server**: `horizon-launcher native-build <folder> --server-output <server-folder>
--version <tag> --trust-native-code`. Players (Horizon or vanilla) install nothing.

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

**Version 1.1.0** rewrites the game side (`src/game.rs`, `src/lib.rs`) for Horizon's mod API; the rules
(`src/rules.rs`) are unchanged.

- The rules pass their 18 tests on hand-built worlds (`cargo test --manifest-path rules/Cargo.toml`).
- The game side compiles, with no clippy warning, against a copy of the mod API's signatures written
  from the Nightly 2026-11-22 sources: the real SDK couldn't be fetched where it was written.
- **Not checked yet**: compiling against the real SDK (`cargo horizon-mod check`), and playing it in the
  game. Until then, treat 1.1.0 as untested.

**Version 1.0.1, in the real game.** Horizon was run without a window, with software rendering, on
Nightly `c9d474d` with the real Minecraft 26.3 data. A player was driven through Horizon's agent control:
it mines logs as a player does, and screenshots were taken. Seen and measured:

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

**Version 1.0.1, outside the game.** The rules passed the same 18 tests; the bundle applied with the
launcher's horizon-mixins, the modded game built with the launcher's `fast` profile, and the `.hnmod`
passed `cargo horizon-mod check`.

**Never checked**: the sound output itself (the test game had no audio device; only the sending was
checked), the fatigue's size, and play with several players.

## Known limits

- During the fall, the tree's leaves have the game's default colour, not the biome's: the game draws
  display blocks that way.
- Twirling leaves take the biome's tint in plains, forests (flower and dark forests included), jungles,
  savannas, swamps (mangrove swamps included), badlands and on beaches. Elsewhere, and in biomes added by
  data packs or mods, they keep the leaves' usual colour: the server doesn't know the biome colour map.
- The whole falling tree is lit like the cut block.
- The log starts shaking at the dig's first crack: a log cut in a blow or two doesn't shake.
- A falling bee nest releases its bees when its lumberjack is in survival or adventure, in the tree's
  dimension. In creative mode its bees stay in, as with vanilla's creative break.
- Shaking a tree doesn't give much: on purpose, so it isn't a farm.

## Build

```sh
cargo horizon-mod check                       # package rules, cargo check against the pinned SDK, what Chop! uses
cargo horizon-mod build                       # target/horizon-mod/chop.hmod, with its uses.json
cargo horizon-mod fits                        # whether it fits the current Nightly
cargo test --manifest-path rules/Cargo.toml   # the rules, without Horizon
```

The first `cargo horizon-mod` command fetches the pinned SDK into `.horizon/`, which is neither in the
repository nor in the package.

| Path | What |
|---|---|
| `horizon-mod.toml` | The manifest: id, version, the build it is made for |
| `src/lib.rs` | The hooks: each moment of the game Chop! listens to |
| `src/game.rs` | The game side: reads and changes the world through the mod API |
| `src/rules.rs` | The rules: the tree, its fall, the physics, the settings (no Horizon) |
| `rules/` | A small crate that tests the rules alone (not in the package) |
| `icon.png` | The icon the launcher and the game's Mods screen show |
| `registry/` | The registry page and its icon |

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
