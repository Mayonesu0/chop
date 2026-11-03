# Submitting Chop! to Horizon's registry

What to put in each field of https://playhorizon.dev/mods/submit (sign in with GitHub as Mayonesu0).

## The file

| Field | Value |
|---|---|
| Kind | Native mod |
| File | `chop-1.0.1.hnmod` (made with `python3 tools/package.py`) |
| SHA-256 | `5abac195c54da7716f35f641489c0dba780444c91b0b3e4f3387ecd4cb2b260b` |
| Id | `chop` |
| Version | `1.0.1` |
| Minecraft version | `26.3` |

What changed:

```
First release (1.0.0 was never published: its package carried a LICENSE file the registry's checker refuses; the code is the same). Pinned to Horizon Nightly 2026-11-01 (c9d474d) and Horizon Mixins ddd9700; the launcher builds those exact revisions.
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

**What it does and doesn't do.** It runs server-side, where the world runs: players who join install nothing. No network access, no telemetry, never runs a program. It reads and writes one file, `<world>/data/chop.txt` (settings, stats, player-placed logs). It only touches trees, the sapling it replants, its own `block_display` entities and what a falling tree hits. Besides the game's functions, it runs a few vanilla commands as the server: `kill` on its own tagged displays, `item modify` for the axe's wear, `damage` on a player the tree hits (off by default), and `summon item` / `setblock … keep` for loot and replanting when the tree is in another dimension.

Native source mod: install it on the launcher's Mods page, press Build there, and confirm the native-code warning. The launcher builds a separate game with it, and it runs with your account's permissions. Each version is pinned to one Horizon Nightly and Horizon Mixins revision, named in its changelog; the launcher builds those exact revisions.

Original mod by Mayonesu0. All rights reserved, except the grants in its [LICENSE](https://github.com/Mayonesu0/chop/blob/main/LICENSE): players may use it unchanged, Horizon's registry may distribute it, and Horizon's reviewers may read it to review it.
```

## Signature (recommended)

Make the key once, on your computer (keep `horizon-mods.key` and its password private and backed up):

```sh
minisign -G -p horizon-mods.pub -s horizon-mods.key
```

Sign the file:

```sh
minisign -S -s horizon-mods.key -m chop-1.0.1.hnmod
```

In the form: the public key is the last line of `horizon-mods.pub` (it starts with `RW`), and the signature is
`chop-1.0.1.hnmod.minisig`.

## Comment on the pull request

The form opens a pull request in horizon-rust/horizon-mods. Horizon's mod policy asks that a licence other than
MIT, Apache-2.0 and the like comes with a written grant for the reviewers, named in the pull request. Post this
comment on it:

```markdown
This replaces #192 (Chop! 1.0.0), which can be closed: its package carried `mixins/chop/LICENSE`, which the checker refused. 1.0.1 has the same code, with the copyright notice as a comment at the top of each source file instead.

Licence: `LicenseRef-Chop-1.0`, all rights reserved with written grants. Its section 3 lets Horizon's reviewers and maintainers read, build, run and test the whole source to review it, and section 2 lets the registry store and distribute unchanged copies: https://github.com/Mayonesu0/chop/blob/main/LICENSE

Chop! is pinned to Horizon c9d474d (Nightly 2026-11-01) and Horizon Mixins ddd9700, because it uses some of the game's own state besides the native hook points (block loot, item spawning, mobs, the LAN hub's sound and particle queues). It was applied with horizon-mixins ddd9700, built with the launcher's `fast` profile and tested in the game on that commit; `cargo horizon-mod check` passes. It does not build on Nightly 2026-11-03 (f42ef42) yet, for a reason outside the mod: horizon-mixins ddd9700 rejects that commit's new `#![recursion_limit = "256"]` in `crates/horizon-client/src/lib.rs` (E_UNSUPPORTED_MACRO), and Horizon's own Zoom fails the same way there.
```
