# Submitting Chop! to Horizon's registry

What to put in each field of https://playhorizon.dev/mods/submit (sign in with GitHub as Mayonesu0).

## The file

| Field | Value |
|---|---|
| Kind | Native mod |
| File | `target/horizon-mod/chop.hmod` (made with `cargo horizon-mod build`, which also packs its `uses.json`) |
| SHA-256 | the line `sha256sum target/horizon-mod/chop.hmod` prints (`Get-FileHash` on Windows) |
| Id | `chop` |
| Version | `1.1.0` |
| Minecraft version | `26.3` |

Before submitting, build it and try it in the game: `cargo horizon-mod check` must pass, and the
comment below says it was tested.

What changed:

```
Rewritten on Horizon's mod API: no mixin and no game internals any more. Made for Horizon Nightly 2026-11-22 (f1bb66f); the launcher also builds it on later Nightlies where nothing it uses has changed. Same features and settings. The log now shakes as the dig's crack grows; a falling bee nest is broken as its lumberjack breaks it, by vanilla's own rules. The package now carries LICENSE and the READMEs.
```

## Its page

| Field | Value |
|---|---|
| Name | `Chop!` |
| Authors | `Mayonesu0` |
| Licence | `LicenseRef-Chop-1.0` |
| Source | `https://github.com/Mayonesu0/chop` |
| Icon | the file `registry/icon.png` (256×256, 2 KiB), or as a link: `https://raw.githubusercontent.com/Mayonesu0/chop/main/registry/icon.png` (a `github.com/…/blob/…` link is a web page, not the image) |

One-line summary:

```
Cut one log and the whole tree tips over, crashes down and breaks apart, with soft vanilla sounds and particles.
```

Description (Markdown):

```markdown
Cut one log with an axe: the whole tree tips over like a real falling tree, crashes to the ground, and breaks apart from the base to the top in little pops. Everything is made of vanilla sounds and particles: no resource pack.

- **The log shakes while you cut it**, then pops when it gives way.
- **A physical fall**: the trunk drops onto the stump, creaks, tips over with gravity and stops where it meets the ground. Leaves in the way are crushed.
- **Falls in front of you**, within an adjustable angle, toward the clearest side.
- **A cute break-up** from the base to the top, with clouds, sparkles and hearts.
- **Replants a sapling** of the tree's kind (from its loot, or always).
- **Chain reactions**: a falling trunk can knock over the next tree.
- **Adjustable damage**: hurts monsters by default; animals, allies and players are spared unless you turn them on.
- Vanilla loot (Fortune, Silk Touch), honest axe wear, fall preview, shaking trees with an empty hand, giant mushrooms and Nether fungi, angry bees, birds, apples, biome-tinted leaves, optional loot into your inventory.
- **Everything is configurable in game**, with commands or by clicking in the chat: `/chop config` (operators), `/chop perso` (each player's sounds, particles and preview), `/chop stats`, `/chop aide`. The menus are in French.

**What it does and doesn't do.** It runs server-side, where the world runs: players who join install nothing. No network access, no telemetry, never runs a program. It reads and writes one file, `<world>/data/chop.txt` (settings, stats, player-placed logs). It only touches trees, the sapling it replants, its own `block_display` entities and what a falling tree hits. It uses Horizon's mod API alone (no mixin, no game internals), and runs a few vanilla commands as the server: `kill` on its own tagged displays, `item modify` for the axe's wear, `damage` on a player the tree hits (off by default), `playsound` / `particle` for one player when players nearby have different personal settings, and `summon item` / `setblock … keep` for loot and replanting when a tree ends while its dimension isn't ticking.

Native source mod: install it on the launcher's Mods page, press Build there, and confirm the native-code warning. The launcher compiles it against the game build's SDK, and it runs with your account's permissions. Each version is made for one Horizon Nightly, named in its changelog; it uses only the mod API, so the launcher also builds it on later Nightlies where nothing it uses has changed.

Original mod by Mayonesu0. All rights reserved, except the grants in its [LICENSE](https://github.com/Mayonesu0/chop/blob/main/LICENSE): players may use it unchanged, Horizon's registry may distribute it, and Horizon's reviewers may read it to review it.
```

## Signature (recommended)

Make the key once, on your computer (keep `horizon-mods.key` and its password private and backed up):

```sh
minisign -G -p horizon-mods.pub -s horizon-mods.key
```

Sign the file:

```sh
minisign -S -s horizon-mods.key -m target/horizon-mod/chop.hmod
```

In the form: the public key is the last line of `horizon-mods.pub` (it starts with `RW`), and the signature is
`target/horizon-mod/chop.hmod.minisig`.

## Comment on the pull request

The form opens a pull request in horizon-rust/horizon-mods. Horizon's mod policy asks that a licence other than
MIT, Apache-2.0 and the like comes with a written grant for the reviewers, named in the pull request. Post this
comment on it:

```markdown
Chop! 1.1.0, an update of Chop! 1.0.1.

Licence: `LicenseRef-Chop-1.0`, all rights reserved with written grants. Its section 3 lets Horizon's reviewers and maintainers read, build, run and test the whole source to review it, and section 2 lets the registry store and distribute unchanged copies: https://github.com/Mayonesu0/chop/blob/main/LICENSE

1.1.0 uses Horizon's mod API alone: no `[native.*]` table, no internals, and its package carries `uses.json`. It is made for Horizon f1bb66f (Nightly 2026-11-22). `cargo horizon-mod check` passes, and it was built by the launcher and tested in the game on that Nightly.
```
