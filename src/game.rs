// Chop! (c) 2026 Mayonesu0. All rights reserved, except the grants in LICENSE: https://github.com/Mayonesu0/chop/blob/main/LICENSE

// Chop! — the game's side (crate::game).
//
// A player cuts a log: on the next tick of that level the tree is read
// (crate::rules decides what it is and which way it falls), its loot is rolled
// with the player's tool, it leaves the level and comes back as block displays that
// turn about their hinge under gravity. Each tick every display gets its new pose,
// eased over one tick; sounds and particles go to the players near it, at their own
// settings; what a block gives drops where it breaks. `/chop` toggles it per player
// and opens the settings, a menu of clickable lines in the chat.
//
// Everything goes through Horizon's mod API (the host): nothing here reaches into the
// game's own code.

use crate::rules::{self, Cell, Config, Key, Phase, Pos, Section, Unit};
use mod_api::text::{TextClick, TextStyle};
use mod_api::{
    Authority, BlockClick, BlockMetadata, BlockShapeKind, BlockState, BlockUpdateFlags, Command,
    CommandArgumentValue, CommandInvocation, CommandSource, Compound, DisplayRef, EntityPlaceEvent,
    GameMode, GameRuleValue, Hand, Host, HostBlockStates, Identifier, ItemStack, LevelRef,
    LifecycleContext, LogicalSide, MobCategory, ParticleKey, ParticleOptions, Placement, PlayerRef,
    Registrar, ServerHost, ServerTickPhase, SoundSource, Tag, Text, TextHover, Transformation,
    WorldPhase,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The mod's id, as Options > Mods knows it.
const MOD: &str = "chop";
/// Every display of the mod carries it.
const TAG: &str = "chop.d";
/// A cut not looked at by then is dropped (the break was cancelled).
const PENDING_TICKS: u32 = 40;
/// A tree whose level hasn't ticked for this long is ended where it is.
const IDLE_TICKS: u64 = 100;
const SAVE_TICKS: u64 = 600;
/// While no tree falls, displays left by a crash are removed this often.
const SWEEP_TICKS: u64 = 600;
/// A dig whose crack hasn't grown for this long has stopped.
const DIG_STALL: u32 = 200;
/// A left click that starts no dig by then (an instant break, creative) shows nothing.
const DIG_START: u32 = 20;

/// `Level.setBlock` flags: the client is told, nothing else happens.
fn quiet() -> BlockUpdateFlags {
    BlockUpdateFlags::CLIENTS
        | BlockUpdateFlags::KNOWN_SHAPE
        | BlockUpdateFlags::SUPPRESS_DROPS
        | BlockUpdateFlags::SKIP_BLOCK_ENTITY_SIDE_EFFECTS
        | BlockUpdateFlags::SKIP_ON_PLACE
}

/// The usual: neighbours and shapes update.
fn normal() -> BlockUpdateFlags {
    BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS
}

/// Mobs that are on the players' side.
const ALLIES: [&str; 6] = [
    "minecraft:iron_golem",
    "minecraft:snow_golem",
    "minecraft:copper_golem",
    "minecraft:villager",
    "minecraft:wandering_trader",
    "minecraft:allay",
];

// ---------------------------------------------------------------------------------
// Particles
// ---------------------------------------------------------------------------------

/// The particles Chop! sends, as the game resolved them when it loaded.
#[derive(Clone, Copy)]
struct Keys {
    block: ParticleKey,
    dust: ParticleKey,
    dust_pillar: ParticleKey,
    tinted_leaves: ParticleKey,
    cherry_leaves: ParticleKey,
    pale_oak_leaves: ParticleKey,
    crimson_spore: ParticleKey,
    warped_spore: ParticleKey,
    poof: ParticleKey,
    happy_villager: ParticleKey,
    crit: ParticleKey,
    heart: ParticleKey,
    cloud: ParticleKey,
}

static KEYS: Mutex<Option<Keys>> = Mutex::new(None);

/// `Hooks::register`: the particle types are asked for once, before any world opens.
pub(crate) fn register(r: &mut dyn Registrar) {
    let keys = Keys {
        block: r.particle_key("minecraft:block"),
        dust: r.particle_key("minecraft:dust"),
        dust_pillar: r.particle_key("minecraft:dust_pillar"),
        tinted_leaves: r.particle_key("minecraft:tinted_leaves"),
        cherry_leaves: r.particle_key("minecraft:cherry_leaves"),
        pale_oak_leaves: r.particle_key("minecraft:pale_oak_leaves"),
        crimson_spore: r.particle_key("minecraft:crimson_spore"),
        warped_spore: r.particle_key("minecraft:warped_spore"),
        poof: r.particle_key("minecraft:poof"),
        happy_villager: r.particle_key("minecraft:happy_villager"),
        crit: r.particle_key("minecraft:crit"),
        heart: r.particle_key("minecraft:heart"),
        cloud: r.particle_key("minecraft:cloud"),
    };
    *KEYS.lock().unwrap_or_else(|e| e.into_inner()) = Some(keys);
}

fn keys() -> Option<Keys> {
    *KEYS.lock().unwrap_or_else(|e| e.into_inner())
}

/// A particle Chop! sends.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Fx {
    /// A block's bits (the state goes with it).
    Block,
    /// Ground dust rising (the state goes with it).
    DustPillar,
    Poof,
    HappyVillager,
    Crit,
    Heart,
    Cloud,
    CherryLeaves,
    PaleOakLeaves,
    CrimsonSpore,
    WarpedSpore,
    /// A falling leaf of this colour (RGB).
    Leaves(u32),
    /// Coloured dust: its colour and size.
    Dust([f32; 3], f32),
}

impl Fx {
    fn options(self, k: &Keys) -> ParticleOptions {
        let key = match self {
            Fx::Block => k.block,
            Fx::DustPillar => k.dust_pillar,
            Fx::Poof => k.poof,
            Fx::HappyVillager => k.happy_villager,
            Fx::Crit => k.crit,
            Fx::Heart => k.heart,
            Fx::Cloud => k.cloud,
            Fx::CherryLeaves => k.cherry_leaves,
            Fx::PaleOakLeaves => k.pale_oak_leaves,
            Fx::CrimsonSpore => k.crimson_spore,
            Fx::WarpedSpore => k.warped_spore,
            Fx::Leaves(_) => k.tinted_leaves,
            Fx::Dust(..) => k.dust,
        };
        let mut o = ParticleOptions::simple(key);
        match self {
            Fx::Leaves(rgb) => {
                o.color = [
                    ((rgb >> 16) & 0xFF) as f32 / 255.0,
                    ((rgb >> 8) & 0xFF) as f32 / 255.0,
                    (rgb & 0xFF) as f32 / 255.0,
                    1.0,
                ];
            }
            Fx::Dust(rgb, scale) => {
                o.color = [rgb[0], rgb[1], rgb[2], 1.0];
                o.scale = scale;
            }
            _ => {}
        }
        o
    }

    /// The particle as `/particle` writes it (`block` names the block for the block kinds).
    fn command(self, block: Option<&str>) -> Option<String> {
        Some(match self {
            Fx::Block => format!("minecraft:block{{block_state:\"{}\"}}", block?),
            Fx::DustPillar => format!("minecraft:dust_pillar{{block_state:\"{}\"}}", block?),
            Fx::Poof => "minecraft:poof".to_owned(),
            Fx::HappyVillager => "minecraft:happy_villager".to_owned(),
            Fx::Crit => "minecraft:crit".to_owned(),
            Fx::Heart => "minecraft:heart".to_owned(),
            Fx::Cloud => "minecraft:cloud".to_owned(),
            Fx::CherryLeaves => "minecraft:cherry_leaves".to_owned(),
            Fx::PaleOakLeaves => "minecraft:pale_oak_leaves".to_owned(),
            Fx::CrimsonSpore => "minecraft:crimson_spore".to_owned(),
            Fx::WarpedSpore => "minecraft:warped_spore".to_owned(),
            // ColorParticleOption: an ARGB int
            Fx::Leaves(rgb) => format!(
                "minecraft:tinted_leaves{{color:{}}}",
                (0xFF00_0000 | rgb) as i32
            ),
            Fx::Dust(c, scale) => format!(
                "minecraft:dust{{color:[{:?}f,{:?}f,{:?}f],scale:{:?}f}}",
                c[0], c[1], c[2], scale
            ),
        })
    }
}

// ---------------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------------

/// Who gets a tree's loot, and whether into their inventory (else at their feet).
type Owner = Option<(u128, bool)>;

struct Pending {
    dim: String,
    pos: Pos,
    state: BlockState,
    player: u128,
    age: u32,
}

/// A tree on its way down.
struct Falling {
    id: u64,
    dim: String,
    /// Who gets the loot at their feet, with that setting.
    owner: Owner,
    fell: rules::Fell,
    /// The display showing each part.
    ents: Vec<Option<DisplayRef>>,
    states: Vec<BlockState>,
    loot: Vec<Vec<ItemStack>>,
    /// Where its displays stand: the cut block's centre.
    anchor: [f64; 3],
    /// Where it touches the ground, and the block it touches.
    contacts: Vec<([f64; 3], BlockState)>,
    /// The mobs it hit (their entity ids), and the players.
    struck: HashSet<u32>,
    struck_players: HashSet<u128>,
    whoosh: bool,
    /// It started to turn (the creak).
    creaked: bool,
    settled: bool,
    leaf: Option<String>,
    /// The cut block's ghost, shaking before it pops.
    ghost: Option<DisplayRef>,
    cut_state: BlockState,
    /// The logs of other trees its trunk comes down on (a chain reaction).
    hits: Vec<Pos>,
    /// 0 for the tree a player cut, 1 for one it knocked down, and so on.
    generation: u32,
    /// How often it fell on after knocking a tree down.
    resumes: u32,
    /// What its loot is rolled with, and whether it has loot.
    tool: Option<ItemStack>,
    drops: bool,
    /// Sounds still to come: the tick (`fell.age`), the sound, where, volume, pitch.
    later: Vec<(u32, &'static str, [f64; 3], f64, f64)>,
    /// What to plant where it stood once it is gone, and whether the saplings came
    /// out of its loot.
    replant: Option<(&'static str, Vec<Pos>, bool)>,
    /// Who felled it (a tree knocked down counts for its feller), for the stats.
    credit: Option<(u128, String)>,
    /// Its leaves' tint (the biome's).
    tint: Option<u32>,
    /// Apples that come loose during the fall: the leaf, the apples, and the tick
    /// after the creak they go.
    apples: Vec<(usize, ItemStack, u32)>,
    /// The tick it started to turn.
    creak_age: Option<u32>,
    /// The farthest part and one halfway up the trunk (where sounds come from).
    tip: usize,
    middle: usize,
    leaves: Vec<usize>,
    last: u64,
}

/// A log a player is digging: it trembles until it breaks.
struct Tremble {
    dim: String,
    pos: Pos,
    /// The face dug, pointing out of the block.
    face: Pos,
    state: BlockState,
    player: u128,
    /// Ticks a shake lasts.
    ticks: u32,
    age: u32,
    display: Option<DisplayRef>,
    /// A few of the tree's leaves, which shiver as it is dug.
    leaves: Vec<Pos>,
    checked: bool,
    /// The player let go.
    stopped: bool,
    /// Ticks since the log broke, waiting for its felling to take the display.
    broken: u32,
    /// A tree shaken by hand, not dug: it shivers a moment and nothing breaks.
    shaking: bool,
    /// Its leaves' tint (the biome's).
    tint: Option<u32>,
    /// Where the trunk will lie: the ground under each log once it has fallen.
    preview: Vec<[f64; 3]>,
    /// The dig's crack (0 to 9) as the game last told it, the first one, and when.
    stage: Option<i32>,
    first_stage: Option<i32>,
    staged_at: u32,
}

/// A player's own settings; -1 follows the world's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Perso {
    /// Sound volume, percent.
    volume: i32,
    /// 0 to 3, as the world's setting.
    particles: i32,
    /// The fall's preview, 0 or 1.
    preview: i32,
}

impl Default for Perso {
    fn default() -> Self {
        Perso {
            volume: -1,
            particles: -1,
            preview: -1,
        }
    }
}

/// What a player has felled.
#[derive(Clone, Debug, Default)]
struct Stats {
    name: String,
    trees: u64,
    logs: u64,
    blocks: u64,
    biggest: u64,
    chain: u32,
}

/// A player shook a tree (a right click with an empty hand on its trunk).
struct ShakeReq {
    dim: String,
    pos: Pos,
    face: Pos,
    player: u128,
}

struct State {
    world: PathBuf,
    config: Config,
    perso: HashMap<u128, Perso>,
    stats: HashMap<u128, Stats>,
    shakes: Vec<ShakeReq>,
    /// When each tree (its foot) and each player last shook.
    shaken: HashMap<(String, Pos), u64>,
    shook: HashMap<u128, u64>,
    placed: HashMap<String, HashSet<Pos>>,
    /// Players who turned Chop! off for themselves.
    off: HashSet<u128>,
    pending: Vec<Pending>,
    trees: Vec<Falling>,
    trembles: Vec<Tremble>,
    rng: rules::Rng,
    session: u64,
    next_id: u64,
    tick: u64,
    swept: HashMap<String, u64>,
    dirty: bool,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);
/// Set while a hook works on the state, so a hook the work sets off waits its turn.
static BUSY: AtomicBool = AtomicBool::new(false);
/// Whether the world runs on a single player's (or LAN host's) own server.
static INTEGRATED: AtomicBool = AtomicBool::new(false);
/// The players' own settings, as the effects read them.
static PERSO: Mutex<Option<HashMap<u128, Perso>>> = Mutex::new(None);

fn perso_snapshot(st: &State) {
    *PERSO.lock().unwrap_or_else(|e| e.into_inner()) = Some(st.perso.clone());
}

fn perso_of(uuid: u128) -> Perso {
    PERSO
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|m| m.get(&uuid).copied())
        .unwrap_or_default()
}

fn seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0xC40B, |d| d.as_nanos() as u64)
}

fn data_file(world: &Path) -> PathBuf {
    world.join("data").join("chop.txt")
}

fn load(world: PathBuf) -> State {
    let text = std::fs::read_to_string(data_file(&world)).unwrap_or_default();
    let mut config = Config::default();
    let mut off = HashSet::new();
    let mut placed: HashMap<String, HashSet<Pos>> = HashMap::new();
    let mut perso: HashMap<u128, Perso> = HashMap::new();
    let mut stats: HashMap<u128, Stats> = HashMap::new();
    let mut section = String::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(head) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = head.trim().to_owned();
            continue;
        }
        if section == "reglages" {
            config.read_line(line);
        } else if section == "sans_chop" {
            if let Ok(u) = u128::from_str_radix(line, 16) {
                off.insert(u);
            }
        } else if let Some(u) = section
            .strip_prefix("perso ")
            .and_then(|h| u128::from_str_radix(h.trim(), 16).ok())
        {
            let p = perso.entry(u).or_default();
            if let Some((k, v)) = line.split_once('=')
                && let Ok(v) = v.trim().parse::<i32>()
            {
                match k.trim() {
                    "sons" => p.volume = v.clamp(-1, 200),
                    "particules" => p.particles = v.clamp(-1, 3),
                    "apercu" => p.preview = v.clamp(-1, 1),
                    _ => {}
                }
            }
        } else if let Some(u) = section
            .strip_prefix("stats ")
            .and_then(|h| u128::from_str_radix(h.trim(), 16).ok())
        {
            let s = stats.entry(u).or_default();
            if let Some((k, v)) = line.split_once('=') {
                let (k, v) = (k.trim(), v.trim());
                let n = v.parse::<u64>().unwrap_or(0);
                match k {
                    "nom" => s.name = v.to_owned(),
                    "arbres" => s.trees = n,
                    "buches" => s.logs = n,
                    "blocs" => s.blocks = n,
                    "record" => s.biggest = n,
                    "chaine" => s.chain = n as u32,
                    _ => {}
                }
            }
        } else if let Some(dim) = section.strip_prefix("posees ") {
            let n: Vec<i32> = line
                .split_whitespace()
                .filter_map(|w| w.parse().ok())
                .collect();
            if let [x, y, z] = n[..] {
                placed
                    .entry(dim.trim().to_owned())
                    .or_default()
                    .insert([x, y, z]);
            }
        }
    }
    let seed = seed();
    *PERSO.lock().unwrap_or_else(|e| e.into_inner()) = Some(perso.clone());
    State {
        world,
        config,
        perso,
        stats,
        shakes: Vec::new(),
        shaken: HashMap::new(),
        shook: HashMap::new(),
        placed,
        off,
        pending: Vec::new(),
        trees: Vec::new(),
        trembles: Vec::new(),
        rng: rules::Rng::new(seed),
        session: seed & 0xFFFF_FFFF,
        next_id: 1,
        tick: 0,
        swept: HashMap::new(),
        dirty: false,
    }
}

fn save(st: &mut State) {
    let mut out = String::from("# Chop! : les données de ce monde\n[reglages]\n");
    for line in st.config.to_lines() {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("[sans_chop]\n");
    let mut off: Vec<&u128> = st.off.iter().collect();
    off.sort();
    for u in off {
        out.push_str(&format!("{u:032x}\n"));
    }
    let mut players: Vec<&u128> = st.perso.keys().collect();
    players.sort();
    for u in players {
        let p = st.perso[u];
        if p == Perso::default() {
            continue;
        }
        out.push_str(&format!(
            "[perso {u:032x}]\nsons = {}\nparticules = {}\napercu = {}\n",
            p.volume, p.particles, p.preview
        ));
    }
    let mut players: Vec<&u128> = st.stats.keys().collect();
    players.sort();
    for u in players {
        let s = &st.stats[u];
        out.push_str(&format!(
            "[stats {u:032x}]\nnom = {}\narbres = {}\nbuches = {}\nblocs = {}\nrecord = {}\nchaine = {}\n",
            s.name, s.trees, s.logs, s.blocks, s.biggest, s.chain
        ));
    }
    let mut dims: Vec<&String> = st.placed.keys().collect();
    dims.sort();
    for dim in dims {
        let mut list: Vec<&Pos> = st.placed[dim].iter().collect();
        if list.is_empty() {
            continue;
        }
        list.sort();
        out.push_str(&format!("[posees {dim}]\n"));
        for p in list {
            out.push_str(&format!("{} {} {}\n", p[0], p[1], p[2]));
        }
    }
    let file = data_file(&st.world);
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = file.with_extension("txt.tmp");
    if std::fs::write(&tmp, out).is_ok() && std::fs::rename(&tmp, &file).is_ok() {
        st.dirty = false;
    }
}

/// Runs `f` on the open world's state (loaded on first use), unless another hook is
/// already working on it.
fn with_state<R>(
    host: &mut dyn ServerHost,
    f: impl FnOnce(&mut dyn ServerHost, &mut State) -> R,
) -> Option<R> {
    let world = host.world_dir()?;
    if BUSY.swap(true, Ordering::Acquire) {
        return None;
    }
    let mut slot = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let mut st = match slot.take() {
        Some(st) if st.world == world => st,
        Some(mut old) => {
            save(&mut old);
            load(world)
        }
        None => load(world),
    };
    drop(slot);
    let r = f(host, &mut st);
    *STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(st);
    BUSY.store(false, Ordering::Release);
    Some(r)
}

// ---------------------------------------------------------------------------------
// Reading the level
// ---------------------------------------------------------------------------------

fn state_at(host: &dyn Host, level: LevelRef, p: Pos) -> Option<BlockState> {
    if p[0].abs() >= 30_000_000 || p[2].abs() >= 30_000_000 {
        return None;
    }
    level.block_state(host, p)
}

/// The block's name, without its properties: `minecraft:oak_log`.
fn name_of(host: &dyn Host, id: BlockState) -> String {
    host.block_states()
        .name(id.block())
        .map(str::to_owned)
        .unwrap_or_default()
}

fn in_tag(host: &dyn Host, id: BlockState, tag: &str) -> bool {
    host.block_in_tag(id.block(), tag).unwrap_or(false)
}

fn prop(host: &dyn Host, id: BlockState, key: &str) -> Option<String> {
    let states = host.block_states();
    id.property(&states, key).ok().flatten().map(str::to_owned)
}

/// The state `id` with property `key` set to `value`.
fn with_prop(host: &dyn Host, id: BlockState, key: &str, value: &str) -> Option<BlockState> {
    let states = host.block_states();
    id.with(&states, key, value).ok()
}

fn is_air(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

/// The collision boxes of `id` at `p`, each `[min x, y, z, max x, y, z]` in the block.
fn collision(host: &dyn Host, level: LevelRef, id: BlockState, p: Pos) -> Option<Vec<[f64; 6]>> {
    level.block_shape(host, id, p, BlockShapeKind::Collision)
}

/// `oak` of `minecraft:stripped_oak_log`.
fn species(name: &str) -> &str {
    let n = name.trim_start_matches("minecraft:");
    let n = n.strip_prefix("stripped_").unwrap_or(n);
    ["_log", "_wood", "_stem", "_hyphae", "_leaves"]
        .iter()
        .find_map(|s| n.strip_suffix(s))
        .unwrap_or(n)
}

/// What hangs on trees, and how.
fn hanger(host: &dyn Host, id: BlockState, name: &str) -> Option<rules::Attach> {
    use rules::Attach::*;
    Some(match name.trim_start_matches("minecraft:") {
        "vine" | "cocoa" | "bee_nest" | "glow_lichen" | "resin_clump" | "creaking_heart" => Side,
        "pale_hanging_moss" | "weeping_vines" | "weeping_vines_plant" => Below,
        "mangrove_propagule" if prop(host, id, "hanging").as_deref() == Some("true") => Below,
        "snow" => Above,
        _ => return None,
    })
}

/// The rules' view of the level.
struct Reader<'g> {
    host: &'g dyn Host,
    level: LevelRef,
    /// The level's lowest block and the one above its highest.
    bounds: Option<(i32, i32)>,
    cells: HashMap<BlockState, Cell>,
    woods: HashMap<String, u16>,
    /// Huge fungi and mushrooms are trees too.
    fungi: bool,
}

/// The blocks of a fungus's or a mushroom's crown.
fn is_cap(name: &str) -> bool {
    matches!(
        name.trim_start_matches("minecraft:"),
        "nether_wart_block"
            | "warped_wart_block"
            | "shroomlight"
            | "red_mushroom_block"
            | "brown_mushroom_block"
    )
}

/// A trunk block: a log, wood, a stem or hyphae, or a mushroom's stem.
fn is_trunk(host: &dyn Host, id: BlockState) -> bool {
    in_tag(host, id, "minecraft:logs") || name_of(host, id) == "minecraft:mushroom_stem"
}

impl<'g> Reader<'g> {
    fn new(host: &'g dyn Host, level: LevelRef, config: &Config) -> Self {
        Reader {
            host,
            level,
            bounds: level.height(host).map(|(min, height)| (min, min + height)),
            cells: HashMap::new(),
            woods: HashMap::new(),
            fungi: config.on(Key::Fungi),
        }
    }
    fn wood(&mut self, name: &str) -> u16 {
        let n = self.woods.len() as u16;
        *self.woods.entry(species(name).to_owned()).or_insert(n + 1)
    }
    fn classify(&mut self, id: BlockState, p: Pos) -> Cell {
        if let Some(c) = self.cells.get(&id) {
            return *c;
        }
        let host = self.host;
        let name = name_of(host, id);
        let cell = if is_air(&name) {
            Cell::Open
        } else if in_tag(host, id, "minecraft:logs")
            || (self.fungi && name == "minecraft:mushroom_stem")
        {
            Cell::Log {
                wood: self.wood(&name),
            }
        } else if self.fungi && is_cap(&name) {
            Cell::Cap
        } else if in_tag(host, id, "minecraft:leaves") {
            Cell::Leaves {
                distance: prop(host, id, "distance")
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(7),
                persistent: prop(host, id, "persistent").as_deref() == Some("true"),
            }
        } else if let Some(a) = hanger(host, id, &name) {
            Cell::Hanger(a)
        } else if host.block_states().replaceable(id) == Some(true) {
            Cell::Open
        } else {
            match collision(host, self.level, id, p) {
                Some(s) if s.is_empty() => Cell::Open,
                // carpets, rails, pressure plates: the tree lies on top of them
                Some(s) if s.iter().all(|b| b[4] <= 0.2) => Cell::Open,
                _ => Cell::Solid,
            }
        };
        self.cells.insert(id, cell);
        cell
    }
}

impl rules::World for Reader<'_> {
    fn cell(&mut self, p: Pos) -> Option<Cell> {
        if p[0].abs() >= 30_000_000 || p[2].abs() >= 30_000_000 {
            return None;
        }
        let (min, max) = self.bounds?;
        if !(min..max).contains(&p[1]) {
            return Some(Cell::Open);
        }
        let id = self.level.block_state(self.host, p)?;
        Some(self.classify(id, p))
    }
}

// ---------------------------------------------------------------------------------
// The players
// ---------------------------------------------------------------------------------

struct Cutter {
    name: String,
    dim: String,
    yaw: f32,
    game_mode: i32,
    sneaking: bool,
    held: Option<ItemStack>,
}

/// The rules' number for a game mode.
fn mode_number(mode: GameMode) -> i32 {
    match mode {
        GameMode::Survival => 0,
        GameMode::Creative => 1,
        GameMode::Adventure => 2,
        GameMode::Spectator => 3,
    }
}

fn dimension_of(host: &dyn Host, player: PlayerRef) -> Option<String> {
    player.dimension(host).map(|d| d.to_string())
}

/// A living player of the server, as the felling needs them.
fn cutter(host: &dyn Host, uuid: u128) -> Option<Cutter> {
    let player = host.player(uuid)?;
    let living = host.living(player.entity())?;
    if living.alive(host) != Some(true) {
        return None;
    }
    Some(Cutter {
        name: player.name(host)?,
        dim: dimension_of(host, player)?,
        yaw: player.entity().rotation(host)?[0],
        game_mode: mode_number(player.game_mode(host)?),
        sneaking: player.entity().is_shift_key_down(host).unwrap_or(false),
        held: living.hand_item(host, Hand::Main),
    })
}

/// An entry of an item's component patch.
fn component<'a>(patch: &'a Compound, key: &str) -> Option<&'a Tag> {
    patch
        .iter()
        .find(|(k, _)| k.to_string_lossy() == key)
        .map(|(_, v)| v)
}

/// The tool's durability, unless it has none or can't break.
fn max_damage(host: &dyn Host, stack: &ItemStack) -> Option<u32> {
    let patch = stack.components().map(|c| c.as_nbt());
    if patch
        .and_then(|p| component(p, "minecraft:unbreakable"))
        .is_some()
    {
        return None;
    }
    let max = match patch.and_then(|p| component(p, "minecraft:max_damage")) {
        Some(Tag::Int(v)) if *v > 0 => *v as u32,
        _ => host.max_damage(stack.item()).unwrap_or(0),
    };
    (max > 0).then_some(max)
}

/// The level of enchantment `id` (`minecraft:unbreaking`) on the stack.
fn enchantment(stack: &ItemStack, id: &str) -> u32 {
    let Some(patch) = stack.components().map(|c| c.as_nbt()) else {
        return 0;
    };
    let Some(Tag::Compound(map)) = component(patch, "minecraft:enchantments") else {
        return 0;
    };
    // worlds older than 1.21.5 wrap the levels in {levels: {...}}
    let map = match component(map, "levels") {
        Some(Tag::Compound(inner)) => inner,
        _ => map,
    };
    let short = id.trim_start_matches("minecraft:");
    map.iter()
        .find(|(k, _)| {
            let k = k.to_string_lossy();
            k == id || k == short
        })
        .and_then(|(_, v)| match v {
            Tag::Byte(l) => Some(i32::from(*l)),
            Tag::Short(l) => Some(i32::from(*l)),
            Tag::Int(l) => Some(*l),
            _ => None,
        })
        .map_or(0, |l| l.max(0) as u32)
}

/// Whether the stack's item is in the item tag `tag`.
fn item_in_tag(host: &dyn Host, stack: &ItemStack, tag: &str) -> bool {
    let Ok(id) = Identifier::parse(tag) else {
        return false;
    };
    let tags = host.tags();
    tags.key(&id)
        .and_then(|key| tags.is_in_tag(stack.item(), key))
        .unwrap_or(false)
}

/// One item `name` (`minecraft:oak_sapling`).
fn stack_of(host: &dyn Host, name: &str) -> Option<ItemStack> {
    let id = Identifier::parse(name).ok()?;
    Some(ItemStack::of(host.item_key(&id)?))
}

/// Whether `uuid` owns this single player world (cheats or not): its server is the
/// window's own and they joined it first.
fn owner(host: &dyn ServerHost, uuid: u128) -> bool {
    INTEGRATED.load(Ordering::Relaxed)
        && host
            .players()
            .server_players()
            .first()
            .is_some_and(|p| p.uuid() == uuid)
}

// ---------------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------------

/// The players of `level`, with where they stand.
fn nearby(host: &dyn Host, level: LevelRef) -> Vec<(PlayerRef, [f64; 3])> {
    level
        .online_players(host)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| Some((p, p.entity().position(host)?)))
        .collect()
}

/// A sound of the game at `at`, to each player near enough, at their own volume
/// (or the world's). When they all hear it alike it is one sound of the level;
/// otherwise each player gets their own (`playsound`).
fn sound(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    name: &str,
    at: [f64; 3],
    volume: f64,
    pitch: f64,
) {
    if volume <= 0.0 {
        return;
    }
    let world = f64::from(config.get(Key::Volume)) / 100.0;
    let pitch = pitch.clamp(0.5, 2.0) as f32;
    // vanilla's reach of a sound: 16 blocks, times its volume above 1
    let reach = |v: f32| f64::from(v.max(1.0)) * 16.0;
    // each player of the level: their volume, and how far they are (squared)
    let players: Vec<(PlayerRef, f32, f64)> = nearby(host, level)
        .into_iter()
        .map(|(p, pos)| {
            let own = perso_of(p.uuid()).volume;
            let scale = if own >= 0 {
                f64::from(own) / 100.0
            } else {
                world
            };
            let d: f64 = (0..3).map(|k| (pos[k] - at[k]).powi(2)).sum();
            (p, (volume * scale) as f32, d)
        })
        .collect();
    let hearers: Vec<(PlayerRef, f32)> = players
        .iter()
        .filter(|(_, v, d)| *d <= reach(*v).powi(2))
        .map(|(p, v, _)| (*p, *v))
        .collect();
    let Some(&(_, first)) = hearers.first() else {
        return;
    };
    // one sound of the level reaches everyone within its reach: it does when they
    // all hear it alike, and no one else hears it
    if hearers.iter().all(|(_, v)| *v == first)
        && players
            .iter()
            .all(|(_, v, d)| *v == first || *d > reach(first).powi(2))
    {
        if first > 0.0 {
            level.play_sound(host, name, at, SoundSource::Blocks, first, pitch);
        }
        return;
    }
    let Some(dim) = level.dimension().id(host) else {
        return;
    };
    for (p, v) in hearers {
        if v <= 0.0 {
            continue;
        }
        let Some(who) = p.name(host) else {
            continue;
        };
        host.commands().run_command(&format!(
            "execute in {dim} run playsound {name} block {who} {:.3} {:.3} {:.3} {v:.3} {pitch:.3}",
            at[0], at[1], at[2]
        ));
    }
}

/// `base` particles at the "normales" setting, to each player within 32 blocks,
/// as many as their own setting (or the world's) says. `only` picks the players.
/// When they all see alike it is one send of the level; otherwise each player gets
/// their own (`particle ... normal <player>`).
#[allow(clippy::too_many_arguments)]
fn particles_to(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    fx: Fx,
    state: Option<BlockState>,
    at: [f64; 3],
    spread: [f64; 3],
    speed: f64,
    base: f64,
    only: &dyn Fn(u128) -> bool,
) {
    const LEVELS: [f64; 4] = [0.0, 0.4, 1.0, 2.0];
    let Some(keys) = keys() else {
        return;
    };
    let world = config.get(Key::Particles).clamp(0, 3);
    let mut viewers: Vec<(PlayerRef, i32)> = Vec::new();
    for (p, pos) in nearby(host, level) {
        // BlockPos.closerToCenterThan(at, 32): from the centre of the player's block
        let d: f64 = (0..3).map(|k| (pos[k].floor() + 0.5 - at[k]).powi(2)).sum();
        if d >= 32.0 * 32.0 {
            continue;
        }
        let count = if only(p.uuid()) {
            let own = perso_of(p.uuid()).particles;
            let setting = if own >= 0 { own.min(3) } else { world };
            let k = LEVELS[setting as usize];
            if k == 0.0 {
                0
            } else {
                ((base * k).round() as i32).max(1)
            }
        } else {
            0
        };
        viewers.push((p, count));
    }
    let Some(&(_, first)) = viewers.first() else {
        return;
    };
    if viewers.iter().all(|(_, c)| *c == first) {
        if first > 0 {
            level.send_particles(host, &fx.options(&keys), state, at, spread, speed, first);
        }
        return;
    }
    // `/particle` shows only to viewers in the level it runs in
    let Some(dim) = level.dimension().id(host) else {
        return;
    };
    let block = state.map(|s| name_of(host, s));
    let Some(particle) = fx.command(block.as_deref()) else {
        return;
    };
    for (p, count) in viewers {
        if count == 0 {
            continue;
        }
        let Some(who) = p.name(host) else {
            continue;
        };
        host.commands().run_command(&format!(
            "execute in {dim} run particle {particle} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {count} normal {who}",
            at[0], at[1], at[2], spread[0], spread[1], spread[2], speed
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn particles(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    fx: Fx,
    state: Option<BlockState>,
    at: [f64; 3],
    spread: [f64; 3],
    speed: f64,
    base: f64,
) {
    particles_to(
        host,
        level,
        config,
        fx,
        state,
        at,
        spread,
        speed,
        base,
        &|_| true,
    );
}

fn block_bits(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    state: BlockState,
    at: [f64; 3],
    base: f64,
) {
    particles(
        host,
        level,
        config,
        Fx::Block,
        Some(state),
        at,
        [0.25, 0.25, 0.25],
        0.1,
        base,
    );
}

fn break_sound(name: &str) -> &'static str {
    let s = species(name);
    let bare = name.trim_start_matches("minecraft:");
    if bare.ends_with("wart_block") {
        "minecraft:block.wart_block.break"
    } else if bare == "shroomlight" {
        "minecraft:block.shroomlight.break"
    } else if bare.starts_with("weeping_vines") {
        "minecraft:block.weeping_vines.break"
    } else if bare.ends_with("mushroom_block") || bare == "mushroom_stem" {
        "minecraft:block.wood.break"
    } else if name.ends_with("_leaves") {
        match s {
            "cherry" => "minecraft:block.cherry_leaves.break",
            "azalea" | "flowering_azalea" => "minecraft:block.azalea_leaves.break",
            _ => "minecraft:block.grass.break",
        }
    } else if s == "cherry" {
        "minecraft:block.cherry_wood.break"
    } else if s == "crimson" || s == "warped" {
        "minecraft:block.stem.break"
    } else if name.ends_with("snow") {
        "minecraft:block.snow.break"
    } else if name.contains("vine") || name.contains("moss") || name.contains("lichen") {
        "minecraft:block.vine.break"
    } else {
        "minecraft:block.wood.break"
    }
}

fn step_sound(leaf: &str) -> &'static str {
    let bare = leaf.trim_start_matches("minecraft:");
    if bare.ends_with("wart_block") {
        return "minecraft:block.wart_block.step";
    }
    if bare.ends_with("mushroom_block") {
        return "minecraft:block.wood.step";
    }
    match species(leaf) {
        "cherry" => "minecraft:block.cherry_leaves.step",
        "azalea" | "flowering_azalea" => "minecraft:block.azalea_leaves.step",
        _ => "minecraft:block.grass.step",
    }
}

// The game's own sounds, layered and tuned: wood that creaks, cracks and thuds,
// leaves that rustle and crash, the soft thud of wool, and bubbly pops.
const WOOL: &str = "minecraft:block.wool.fall";
const PLOP: &str = "minecraft:entity.chicken.egg";
const BUBBLE: &str = "minecraft:block.bubble_column.bubble_pop";
const CREAK: &str = "minecraft:block.wooden_door.open";
const CRACK: &str = "minecraft:block.wood.break";
const THUD: &str = "minecraft:block.wood.fall";

/// Whether a leaf block's colour is its biome's (oak, jungle, acacia, dark oak).
fn biome_tinted(leaf: &str) -> bool {
    matches!(species(leaf), "oak" | "jungle" | "acacia" | "dark_oak")
}

/// The foliage colour of the biome at `p`, for the biomes where trees of those kinds
/// grow (vanilla's colormap at their climate, or their own foliage colour). Others get
/// the leaves' own colour.
fn foliage_at(host: &dyn Host, level: LevelRef, p: Pos) -> Option<u32> {
    let biome = level.biome_at(host, p)?;
    Some(match biome.trim_start_matches("minecraft:") {
        "plains" | "sunflower_plains" | "beach" => 0x77AB2F,
        "forest" | "flower_forest" | "dark_forest" => 0x59AE30,
        "jungle" | "bamboo_jungle" => 0x30BB0B,
        "sparse_jungle" => 0x3EB80F,
        "savanna" | "savanna_plateau" | "windswept_savanna" => 0xAEA42A,
        "swamp" => 0x6A7039,
        "mangrove_swamp" => 0x8DB127,
        "badlands" | "wooded_badlands" | "eroded_badlands" => 0x9E814D,
        _ => return None,
    })
}

/// The tint of the leaves of a tree at `p`: its biome's for the leaves that take
/// it, else none (their own colour).
fn tint_for(host: &dyn Host, level: LevelRef, leaf: Option<&str>, p: Pos) -> Option<u32> {
    leaf.filter(|l| biome_tinted(l))
        .and_then(|_| foliage_at(host, level, p))
}

/// The falling-leaf particle of a leaf block, tinted as its foliage is (`tint`,
/// the biome's, for the leaves that take it); spores for a fungus's wart, none
/// for a mushroom's cap.
fn leaf_particle(leaf: &str, tint: Option<u32>) -> Option<Fx> {
    let bare = leaf.trim_start_matches("minecraft:");
    if bare == "nether_wart_block" {
        return Some(Fx::CrimsonSpore);
    }
    if bare == "warped_wart_block" {
        return Some(Fx::WarpedSpore);
    }
    if is_cap(bare) {
        return None;
    }
    let fixed = match species(leaf) {
        "cherry" => return Some(Fx::CherryLeaves),
        "pale_oak" => return Some(Fx::PaleOakLeaves),
        "spruce" => 0x619961,
        "birch" => 0x80A755,
        "jungle" => 0x30BB0B,
        "acacia" => 0xAEA42A,
        "mangrove" => 0x8DB127,
        "azalea" | "flowering_azalea" => 0x6E9A35,
        _ => 0x59AE30,
    };
    Some(Fx::Leaves(
        tint.filter(|_| biome_tinted(leaf)).unwrap_or(fixed),
    ))
}

// ---------------------------------------------------------------------------------
// Displays
// ---------------------------------------------------------------------------------

fn floats(v: &[f32]) -> Tag {
    Tag::List(v.iter().map(|x| Tag::Float(x.to_bits())).collect())
}

/// `minecraft:oak_log[axis=y]` as its name and its properties.
fn split_state(text: &str) -> (&str, Vec<(&str, &str)>) {
    let Some((name, rest)) = text.split_once('[') else {
        return (text, Vec::new());
    };
    let props = rest
        .trim_end_matches(']')
        .split(',')
        .filter_map(|kv| kv.split_once('='))
        .collect();
    (name, props)
}

#[allow(clippy::too_many_arguments)]
fn summon_display(
    host: &mut dyn ServerHost,
    dim: &str,
    anchor: [f64; 3],
    state: BlockState,
    translation: [f32; 3],
    scale: f32,
    session: u64,
    tree: u64,
) -> Option<DisplayRef> {
    let text = host.block_state_string(state)?;
    let (name, props) = split_state(&text);
    let mut block = Compound::new();
    block.insert("Name".into(), Tag::String(name.into()));
    let mut properties = Compound::new();
    for (k, v) in props {
        properties.insert(k.into(), Tag::String(v.into()));
    }
    block.insert("Properties".into(), Tag::Compound(properties));
    let mut t = Compound::new();
    t.insert("translation".into(), floats(&translation));
    t.insert("left_rotation".into(), floats(&[0.0, 0.0, 0.0, 1.0]));
    t.insert("scale".into(), floats(&[scale; 3]));
    t.insert("right_rotation".into(), floats(&[0.0, 0.0, 0.0, 1.0]));
    let mut c = Compound::new();
    c.insert("block_state".into(), Tag::Compound(block));
    c.insert("transformation".into(), Tag::Compound(t));
    // each new pose eases over one tick
    c.insert("interpolation_duration".into(), Tag::Int(1));
    c.insert("start_interpolation".into(), Tag::Int(0));
    c.insert("teleport_duration".into(), Tag::Int(0));
    c.insert("view_range".into(), Tag::Float(1.5f32.to_bits()));
    let tags = [
        TAG.to_owned(),
        format!("chop.s{session}"),
        format!("chop.t{tree}"),
    ];
    c.insert(
        "Tags".into(),
        Tag::List(
            tags.iter()
                .map(|t| Tag::String(t.as_str().into()))
                .collect(),
        ),
    );
    let id = host.summon(dim, "minecraft:block_display", anchor, c)?;
    let entity = host.entity(id)?;
    host.display(entity)
}

/// A display's next pose: its transformation is set and the players who see it ease
/// toward it over one tick (the interpolation restarts).
fn pose(
    host: &mut dyn ServerHost,
    display: DisplayRef,
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: f32,
) {
    let t = Transformation {
        translation,
        left_rotation: rotation,
        scale: [scale; 3],
        ..Transformation::IDENTITY
    };
    if display.set_transformation(host, t) {
        display.set_start_interpolation(host, 0);
    }
}

fn remove(host: &mut dyn ServerHost, display: DisplayRef) {
    display.entity().discard(host);
}

// ---------------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------------

/// `break_block`: a player breaks a block. A log is looked at on the next tick of
/// its level, once the game broke it.
pub(crate) fn destroy(host: &mut dyn ServerHost, pos: Pos, state: BlockState) {
    if !host.on(MOD) || !is_trunk(host, state) {
        return;
    }
    let Some(player) = host.players().acting_player() else {
        return;
    };
    let Some(dim) = dimension_of(host, player) else {
        return;
    };
    let uuid = player.uuid();
    with_state(host, |_, st| {
        if let Some(set) = st.placed.get_mut(&dim)
            && set.remove(&pos)
        {
            // a placed log goes: it is never a tree's
            st.dirty = true;
            return;
        }
        st.pending.push(Pending {
            dim,
            pos,
            state,
            player: uuid,
            age: 0,
        });
    });
}

/// `left_click_block`: a player starts digging a block. A log trembles while it is
/// dug, unless it breaks at the first blow.
pub(crate) fn left_click(host: &mut dyn ServerHost, pos: Pos, face: Pos, state: BlockState) {
    if !host.on(MOD) || !is_trunk(host, state) {
        return;
    }
    let Some(player) = host.players().acting_player() else {
        return;
    };
    let Some(dim) = dimension_of(host, player) else {
        return;
    };
    let uuid = player.uuid();
    with_state(host, |_, st| {
        if !st.config.on(Key::Shake) || st.off.contains(&uuid) {
            return;
        }
        for t in st.trembles.iter_mut().filter(|t| t.player == uuid) {
            t.stopped = true;
        }
        st.trembles.push(Tremble {
            dim,
            pos,
            face,
            state,
            player: uuid,
            ticks: 0,
            age: 0,
            display: None,
            leaves: Vec::new(),
            checked: false,
            stopped: false,
            broken: 0,
            shaking: false,
            tint: None,
            preview: Vec::new(),
            stage: None,
            first_stage: None,
            staged_at: 0,
        });
    });
}

/// `block_break_progress_update`: the crack of a player's dig grew.
pub(crate) fn dig_progress(host: &mut dyn ServerHost, player: u128, pos: Pos, stage: i32) {
    with_state(host, |_, st| {
        if let Some(t) = st
            .trembles
            .iter_mut()
            .find(|t| t.player == player && t.pos == pos && !t.shaking && !t.stopped)
        {
            t.first_stage.get_or_insert(stage);
            t.stage = Some(stage);
            t.staged_at = t.age;
        }
    });
}

/// `block_damage_abort`: a player stopped digging; their log is let be.
pub(crate) fn dig_stopped(host: &mut dyn ServerHost, player: u128, pos: Pos) {
    with_state(host, |_, st| {
        for t in st
            .trembles
            .iter_mut()
            .filter(|t| t.player == player && t.pos == pos && !t.shaking)
        {
            t.stopped = true;
        }
    });
}

/// `right_click_block`, on the server: a right click with an empty hand on a trunk
/// shakes the tree.
pub(crate) fn right_click(host: &mut dyn ServerHost, click: &BlockClick) {
    if !matches!(click.side, LogicalSide::Server)
        || !matches!(click.hand, Hand::Main)
        || click.stack.is_some()
        || !host.on(MOD)
    {
        return;
    }
    let Some(player) = host.player(click.player) else {
        return;
    };
    let Some(level) = player.level(host) else {
        return;
    };
    if !state_at(host, level, click.pos).is_some_and(|id| is_trunk(host, id)) {
        return;
    }
    let Some(dim) = level.dimension().id(host) else {
        return;
    };
    let (pos, face, uuid) = (click.pos, click.hit.face, click.player);
    with_state(host, |_, st| {
        if st.config.on(Key::ShakeTree) && st.config.on(Key::Active) {
            st.shakes.push(ShakeReq {
                dim,
                pos,
                face,
                player: uuid,
            });
        }
    });
}

/// `entity_place`: logs a player places are remembered and never felled.
pub(crate) fn place(host: &mut dyn ServerHost, event: &EntityPlaceEvent) {
    if !host.on(MOD) {
        return;
    }
    let Some(player) = host.players().acting_player() else {
        return;
    };
    if event.placer.is_some_and(|e| e != player.entity()) {
        return;
    }
    let mut logs: Vec<Pos> = Vec::new();
    if is_trunk(host, event.state) {
        logs.push([event.pos.x, event.pos.y, event.pos.z]);
    }
    if let Placement::Multi(blocks) = &event.placement {
        for b in blocks {
            let at = [b.pos.x, b.pos.y, b.pos.z];
            if is_trunk(host, b.state) && !logs.contains(&at) {
                logs.push(at);
            }
        }
    }
    if logs.is_empty() {
        return;
    }
    let Some(dim) = dimension_of(host, player) else {
        return;
    };
    with_state(host, |_, st| {
        if !st.config.on(Key::Protect) {
            return;
        }
        let set = st.placed.entry(dim).or_default();
        for p in logs {
            if set.insert(p) {
                st.dirty = true;
            }
        }
    });
}

/// `server_tick`: the clock, cuts that waited too long, idle trees, saving.
pub(crate) fn server_tick(
    host: &mut dyn ServerHost,
    context: &LifecycleContext,
    phase: ServerTickPhase,
) {
    INTEGRATED.store(
        matches!(context.authority, Authority::IntegratedServer),
        Ordering::Relaxed,
    );
    let ServerTickPhase::ServerTickStart { world_running, .. } = phase else {
        return;
    };
    with_state(host, |host, st| {
        st.tick += 1;
        for p in st.pending.iter_mut() {
            p.age += 1;
        }
        st.pending.retain(|p| p.age <= PENDING_TICKS);
        if world_running {
            // a level nobody is in any more stops ticking: its trees end where they are
            let mut i = 0;
            while i < st.trees.len() {
                if st.tick.saturating_sub(st.trees[i].last) > IDLE_TICKS {
                    let mut tree = st.trees.remove(i);
                    end_tree(host, &mut tree);
                } else {
                    i += 1;
                }
            }
        } else {
            // a frozen world holds them
            let now = st.tick;
            for t in st.trees.iter_mut() {
                t.last = now;
            }
        }
        if st.dirty && st.tick % SAVE_TICKS == 0 {
            save(st);
        }
    });
}

/// `level_tick_pre`: the level is in place. Its trees move a tick, then the cuts
/// made in it are looked at.
pub(crate) fn level_tick(host: &mut dyn ServerHost, level: LevelRef) {
    let Some(dimension) = level.dimension().id(host) else {
        return;
    };
    let dimension = dimension.as_str();
    let enabled = host.on(MOD);
    with_state(host, |host, st| {
        if !enabled {
            // switched off in Options > Mods: what falls here ends at once
            let (here, rest): (Vec<Falling>, Vec<Falling>) = std::mem::take(&mut st.trees)
                .into_iter()
                .partition(|t| t.dim == dimension);
            st.trees = rest;
            for mut t in here {
                end_tree(host, &mut t);
            }
            st.pending.retain(|p| p.dim != dimension);
            return;
        }
        let config = st.config.clone();
        let mut knocks = Vec::new();
        let mut i = 0;
        while i < st.trees.len() {
            if st.trees[i].dim != dimension {
                i += 1;
                continue;
            }
            st.trees[i].last = st.tick;
            let (done, knock) = tick_tree(host, level, &config, &mut st.rng, &mut st.trees[i]);
            knocks.extend(knock);
            if done {
                st.trees.remove(i);
            } else {
                i += 1;
            }
        }
        for k in knocks {
            chain(host, level, st, dimension, k);
        }
        let mut started = 0;
        let mut j = 0;
        while j < st.pending.len() && started < 3 {
            if st.pending[j].dim != dimension {
                j += 1;
                continue;
            }
            let p = st.pending.remove(j);
            match job(host, level, st, &p) {
                Job::Retry => {
                    st.pending.insert(j, p);
                    j += 1;
                }
                Job::Done => started += 1,
            }
        }
        tick_shakes(host, level, st, dimension);
        tick_trembles(host, level, st, dimension);
        // displays a crash left behind, while nothing falls
        let last = st.swept.get(dimension).copied();
        if st.trees.is_empty()
            && st.trembles.is_empty()
            && last.is_none_or(|t| st.tick.saturating_sub(t) >= SWEEP_TICKS)
        {
            st.swept.insert(dimension.to_owned(), st.tick);
            host.commands().run_command(&format!(
                "execute in {dimension} run kill @e[type=minecraft:block_display,tag={TAG}]"
            ));
        }
    });
}

/// `server_started`: a world opened, its data loads.
pub(crate) fn opened(host: &mut dyn ServerHost) {
    let Some(world) = host.world_dir() else {
        return;
    };
    let mut slot = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(mut old) = slot.take() {
        save(&mut old);
    }
    *slot = Some(load(world));
}

/// `server_stopping`: what still falls drops at once.
pub(crate) fn stopping(host: &mut dyn ServerHost) {
    with_state(host, |host, st| {
        for mut t in std::mem::take(&mut st.trees) {
            end_tree(host, &mut t);
        }
        st.pending.clear();
        save(st);
    });
}

/// `world_lifecycle`: a world closed (or didn't open), its data is saved.
pub(crate) fn lifecycle(_host: &mut dyn ServerHost, phase: WorldPhase) {
    match phase {
        WorldPhase::WorldCloseFinal
        | WorldPhase::WorldCloseFailed
        | WorldPhase::WorldOpenFailed => {
            let mut slot = STATE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(mut old) = slot.take() {
                save(&mut old);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------------
// A cut
// ---------------------------------------------------------------------------------

enum Job {
    Done,
    /// The log is still there: the break hasn't happened yet.
    Retry,
}

fn action_bar(host: &mut dyn ServerHost, player: u128, text: &str) {
    if let Some(p) = host.player(player) {
        let _ = p.action_bar(host, span(text, "gold"));
    }
}

/// Where a felling starts from: a player's cut, or a trunk knocked down.
struct Start {
    dim: String,
    cut: Pos,
    cut_state: BlockState,
    /// Which way it should fall (degrees), and how far it may turn from there.
    yaw: f64,
    aim: bool,
    cone: i32,
    tool: Option<ItemStack>,
    drops: bool,
    /// Who gets the loot at their feet, with that setting.
    owner: Owner,
    /// The player who cut it (their uuid and name), for messages and the wear.
    cutter: Option<(u128, String)>,
    /// Who the felling counts for: the cutter, or who felled the tree that
    /// knocked this one down.
    credit: Option<(u128, String)>,
    wear: bool,
    /// A player's own cut out of creative: it tires them (with that setting).
    tired: bool,
    /// The display of the log that trembled as it was dug: it puffs up and pops.
    ghost: Option<(DisplayRef, Pos)>,
    generation: u32,
    /// The cut log is still in the level (a trunk knocked down): it breaks once
    /// the tree behind it is known.
    standing: bool,
}

fn job(host: &mut dyn ServerHost, level: LevelRef, st: &mut State, p: &Pending) -> Job {
    if state_at(host, level, p.pos).is_some_and(|id| is_trunk(host, id)) {
        return Job::Retry;
    }
    // the log that trembled as it was dug, if it did
    let ghost = st
        .trembles
        .iter()
        .position(|t| t.dim == p.dim && t.pos == p.pos)
        .map(|i| st.trembles.remove(i))
        .and_then(|t| t.display.map(|d| (d, t.face)));
    if !felled(host, level, st, p, ghost)
        && let Some((d, _)) = ghost
    {
        remove(host, d);
    }
    Job::Done
}

/// Whether the game drops what blocks give (the `block_drops` rule).
fn block_drops(host: &dyn ServerHost) -> bool {
    !matches!(
        host.game_rule("minecraft:block_drops"),
        Ok(GameRuleValue::Bool(false))
    )
}

/// Fells the tree of the log `p` broke, when its cutter may: whether it did.
fn felled(
    host: &mut dyn ServerHost,
    level: LevelRef,
    st: &mut State,
    p: &Pending,
    ghost: Option<(DisplayRef, Pos)>,
) -> bool {
    if st.off.contains(&p.player) {
        return false;
    }
    let Some(who) = cutter(host, p.player) else {
        return false;
    };
    if who.dim != p.dim {
        return false;
    }
    let axe = who
        .held
        .as_ref()
        .is_some_and(|s| item_in_tag(host, s, "minecraft:axes"));
    let config = &st.config;
    if !rules::fells(config, who.game_mode, who.sneaking, axe) {
        return false;
    }
    let creative = who.game_mode == 1;
    let start = Start {
        dim: p.dim.clone(),
        cut: p.pos,
        cut_state: p.state,
        yaw: f64::from(who.yaw),
        aim: config.on(Key::Aim),
        cone: config.get(Key::Cone),
        tool: who.held.clone(),
        drops: !creative && block_drops(host),
        owner: match config.get(Key::Loot) {
            1 => Some((p.player, false)),
            2 => Some((p.player, true)),
            _ => None,
        },
        cutter: Some((p.player, who.name.clone())),
        credit: Some((p.player, who.name.clone())),
        wear: config.on(Key::Wear) && !creative,
        tired: !creative,
        ghost,
        generation: 0,
        standing: false,
    };
    fell_tree(host, level, st, start)
}

/// What the block at `p` gives, rolled with `tool`, while it is still there.
fn loot_at(
    host: &mut dyn ServerHost,
    level: LevelRef,
    p: Pos,
    tool: Option<&ItemStack>,
) -> Vec<ItemStack> {
    level
        .block_drops(host, p, tool, None)
        .ok()
        .flatten()
        .unwrap_or_default()
}

/// Reads the tree behind `s.cut`, takes it out of the level and starts its fall:
/// false when there is no tree to fell there.
fn fell_tree(host: &mut dyn ServerHost, level: LevelRef, st: &mut State, s: Start) -> bool {
    let config = st.config.clone();
    let cut_name = name_of(host, s.cut_state);
    let tree = {
        let placed = st.placed.get(&s.dim);
        let protect = config.on(Key::Protect);
        let is_placed = |q: Pos| protect && placed.is_some_and(|set| set.contains(&q));
        let mut reader = Reader::new(host, level, &config);
        let wood = reader.wood(&cut_name);
        let mut look = rules::Look::new(&mut reader);
        rules::scan(
            &mut look,
            &is_placed,
            s.cut,
            wood,
            &rules::Limits::of(&config),
        )
    };
    let tree = match tree {
        Ok(t) => t,
        Err(rules::Refusal::TooBig) => {
            if let Some((uuid, _)) = &s.cutter {
                action_bar(
                    host,
                    *uuid,
                    "🪓 Chop! Cet arbre est trop grand : il se coupe bûche par bûche.",
                );
            }
            return false;
        }
        Err(_) => return false,
    };

    // the axe pays for every log
    if s.wear
        && let Some((uuid, name)) = &s.cutter
        && let Some(stack) = &s.tool
        && let Some(max) = max_damage(host, stack)
    {
        let n = rules::wear(
            tree.logs.len() as u32,
            enchantment(stack, "minecraft:unbreaking"),
            &mut st.rng,
        );
        let after = stack.damage() + n;
        if after >= max {
            action_bar(
                host,
                *uuid,
                &format!("🪓 Chop! Ta hache est trop usée pour cet arbre ({n} points de plus)."),
            );
            return false;
        }
        if n > 0 {
            host.commands().run_command(&format!(
                "item modify entity {name} weapon.mainhand {{type:\"minecraft:set_components\",components:{{\"minecraft:damage\":{after}}}}}"
            ));
        }
    }

    let dim = s.dim.clone();
    let center = |c: Pos| [c[0] as f64 + 0.5, c[1] as f64 + 0.5, c[2] as f64 + 0.5];
    let cut = center(s.cut);
    // a trunk knocked down breaks at its foot
    if s.standing {
        let items = if s.drops {
            loot_at(host, level, s.cut, s.tool.as_ref())
        } else {
            Vec::new()
        };
        clear(host, level, s.cut, s.cut_state, normal());
        block_bits(host, level, &config, s.cut_state, cut, 10.0);
        for item in items {
            drop_item(host, level, &mut st.rng, cut, item, s.owner);
        }
    }

    // which way, how far, what it crushes, whose trunks it comes down on
    let animate = config.on(Key::Animation);
    let (body, contact, crush, hits) = {
        let mut reader = Reader::new(host, level, &config);
        let mut look = rules::Look::new(&mut reader);
        let sink = if animate {
            rules::gap(&mut look, &tree)
        } else {
            0.0
        };
        let dir = if animate {
            rules::choose_dir(&tree, &mut look, s.yaw, s.aim, s.cone, sink)
        } else {
            rules::yaw_dir(s.yaw)
        };
        let body = rules::Body::new(&tree, dir, sink);
        if animate {
            let contact = body.contact(&mut look, true);
            let crush = body.crush(&mut look, contact.theta);
            let hits = log_hits(&mut look, &contact);
            (body, contact, crush, hits)
        } else {
            let none = rules::Contact {
                theta: 0.0,
                ground: true,
                points: Vec::new(),
            };
            (body, none, Vec::new(), Vec::new())
        }
    };

    // its loot, rolled with the tool while the blocks are still there
    let mut states = Vec::with_capacity(body.parts.len());
    let mut loot = Vec::with_capacity(body.parts.len());
    for part in &body.parts {
        let id = state_at(host, level, part.cell).unwrap_or(s.cut_state);
        states.push(id);
        loot.push(if s.drops {
            loot_at(host, level, part.cell, s.tool.as_ref())
        } else {
            Vec::new()
        });
    }
    let contacts = dust_points(host, level, &contact);
    let leaf = body
        .parts
        .iter()
        .position(|part| part.leaf)
        .map(|i| name_of(host, states[i]));

    let tint = tint_for(host, level, leaf.as_deref(), s.cut);
    credit_felling(st, &s, &tree);
    if s.tired
        && config.on(Key::Fatigue)
        && let Some((uuid, _)) = &s.credit
        && let Some(p) = host.player(*uuid)
    {
        // felling makes the lumberjack a little hungry
        p.add_exhaustion(host, 0.15 * tree.logs.len() as f32);
    }

    // what hangs on it: its loot too
    let mut hung = Vec::with_capacity(tree.hangers.len());
    for h in &tree.hangers {
        if let Some(id) = state_at(host, level, *h) {
            let items = if s.drops {
                loot_at(host, level, *h, s.tool.as_ref())
            } else {
                Vec::new()
            };
            hung.push((*h, id, items));
        }
    }
    // the saplings that go back where it stood
    let replanting = plan_replant(
        host,
        level,
        &config,
        &tree,
        &cut_name,
        leaf.as_deref(),
        s.drops,
        &mut loot,
        &mut hung,
    );

    // what hangs on it breaks off first
    // the player a nest's bees come out for: its feller, here and breaking blocks as
    // survival players do (a creative player's break keeps them in, as in vanilla)
    let lumberjack = s
        .credit
        .as_ref()
        .and_then(|(uuid, _)| host.player(*uuid))
        .filter(|p| p.level(host) == Some(level))
        .filter(|p| {
            matches!(
                p.game_mode(host),
                Some(GameMode::Survival | GameMode::Adventure)
            )
        });
    for (n, (h, id, items)) in hung.into_iter().enumerate() {
        let at = center(h);
        let name = name_of(host, id);
        let nest = matches!(name.as_str(), "minecraft:bee_nest" | "minecraft:beehive");
        if nest
            && config.on(Key::Bees)
            && let Some(p) = lumberjack
            && p.destroy_block(host, h, matches!(s.owner, Some((_, true))))
        {
            // broken as the player breaks it: without Silk Touch its bees come out,
            // angry, as in vanilla; its loot is the game's
            continue;
        }
        clear(host, level, h, id, normal());
        block_bits(host, level, &config, id, at, 4.0);
        if n < 3 {
            sound(host, level, &config, break_sound(&name), at, 0.6, 1.2);
        }
        for item in items {
            drop_item(host, level, &mut st.rng, at, item, None);
        }
    }
    // then the tree leaves the level, leaves first
    for i in (0..body.parts.len()).rev() {
        let part = &body.parts[i];
        let flags = if part.surface { normal() } else { quiet() };
        clear(host, level, part.cell, states[i], flags);
    }

    if !animate {
        if let Some((ghost, _)) = s.ghost {
            remove(host, ghost);
        }
        // everything breaks where it stood, with a little tune
        let every = (body.parts.len() / 150).max(1);
        for (i, part) in body.parts.iter().enumerate() {
            let at = center(part.cell);
            if i % every == 0 {
                block_bits(host, level, &config, states[i], at, 3.0);
                particles(
                    host,
                    level,
                    &config,
                    Fx::Poof,
                    None,
                    at,
                    [0.2, 0.2, 0.2],
                    0.02,
                    1.0,
                );
            }
            for item in std::mem::take(&mut loot[i]) {
                drop_item(host, level, &mut st.rng, at, item, s.owner);
            }
        }
        sound(host, level, &config, CRACK, cut, 0.9, 0.8);
        if let Some(l) = &leaf {
            sound(host, level, &config, break_sound(l), cut, 0.8, 0.9);
        }
        sound(host, level, &config, PLOP, cut, 0.9, 1.2);
        sound(host, level, &config, BUBBLE, cut, 0.8, 1.0);
        sound(host, level, &config, BUBBLE, cut, 0.6, 1.4);
        if let Some((sapling, spots, from_loot)) = replanting {
            replant(
                host,
                level,
                &config,
                &mut st.rng,
                sapling,
                &spots,
                from_loot,
            );
        }
        return true;
    }

    // the falling tree, and the ghost of the cut block shaking under it
    let id = st.next_id;
    st.next_id += 1;
    let anchor = cut;
    let mut ents = Vec::with_capacity(body.parts.len());
    for (i, part) in body.parts.iter().enumerate() {
        // where the block stood: the body is planned dropped, it shows upright first
        let tr = [0, 1, 2].map(|k| (f64::from(part.cell[k]) - anchor[k]) as f32);
        ents.push(summon_display(
            host, &dim, anchor, states[i], tr, 1.0, st.session, id,
        ));
    }
    let ghost = s.ghost.map(|(ghost, face)| {
        // the log that trembled swells one last time; it pops on the next tick
        let (tr, scale) = rules::tremble_pose(0, 1.0, face);
        let tr = tr.map(|v| v - 0.06);
        pose(host, ghost, tr, [0.0, 0.0, 0.0, 1.0], scale + 0.12);
        ghost
    });
    if ghost.is_none() {
        // cut in one blow, or pushed over: it goes at once
        sound(
            host,
            level,
            &config,
            "minecraft:block.wood.hit",
            cut,
            0.6,
            1.3,
        );
    }
    let tip = (0..body.parts.len())
        .max_by(|a, b| body.parts[*a].r.total_cmp(&body.parts[*b].r))
        .unwrap_or(0);
    let logs: Vec<usize> = (0..body.parts.len())
        .filter(|i| !body.parts[*i].leaf)
        .collect();
    let middle = logs.get(logs.len() / 2).copied().unwrap_or(tip);
    let leaves: Vec<usize> = (0..body.parts.len())
        .filter(|i| body.parts[*i].leaf)
        .collect();
    let motion = rules::Motion::new(&body, &contact, config.get(Key::Speed));
    let lie = (config.get(Key::Rest) * 2).max(0) as u32;
    // the apples come loose as it goes
    let mut apples = Vec::new();
    if config.on(Key::Apples)
        && let Some(apple) = Identifier::parse("minecraft:apple")
            .ok()
            .and_then(|id| host.item_key(&id))
    {
        for &i in &leaves {
            let (fruit, kept): (Vec<ItemStack>, Vec<ItemStack>) = std::mem::take(&mut loot[i])
                .into_iter()
                .partition(|stack| stack.item() == apple);
            loot[i] = kept;
            for stack in fruit {
                let when = 3 + st.rng.below(10);
                apples.push((i, stack, when));
            }
        }
    }
    st.trees.push(Falling {
        id,
        dim,
        owner: s.owner,
        fell: rules::Fell::new(body, motion, crush, lie, false, config.get(Key::Speed)),
        ents,
        states,
        loot,
        anchor,
        contacts,
        struck: HashSet::new(),
        struck_players: HashSet::new(),
        whoosh: false,
        creaked: false,
        settled: false,
        leaf,
        ghost,
        cut_state: s.cut_state,
        hits,
        generation: s.generation,
        resumes: 0,
        tool: s.tool,
        drops: s.drops,
        later: Vec::new(),
        replant: replanting,
        credit: s.credit,
        tint,
        apples,
        creak_age: None,
        tip,
        middle,
        leaves,
        last: st.tick,
    });
    true
}

/// A felling counts in its feller's stats.
fn credit_felling(st: &mut State, s: &Start, tree: &rules::Tree) {
    let Some((uuid, name)) = &s.credit else {
        return;
    };
    let stats = st.stats.entry(*uuid).or_default();
    stats.name = name.clone();
    stats.trees += 1;
    stats.logs += tree.logs.len() as u64 + 1;
    stats.blocks += tree.blocks() as u64 + 1;
    stats.biggest = stats.biggest.max(tree.blocks() as u64 + 1);
    stats.chain = stats.chain.max(s.generation);
    st.dirty = true;
}

/// The logs of other trees among the cells the trunk touches.
fn log_hits(look: &mut rules::Look<'_>, contact: &rules::Contact) -> Vec<Pos> {
    let mut out: Vec<Pos> = Vec::new();
    for (_, cell) in &contact.points {
        if matches!(look.cell(*cell), Some(Cell::Log { .. })) && !out.contains(cell) {
            out.push(*cell);
        }
    }
    out
}

/// Up to ten points where the trunk touches, with the block there (its dust).
fn dust_points(
    host: &dyn Host,
    level: LevelRef,
    contact: &rules::Contact,
) -> Vec<([f64; 3], BlockState)> {
    contact
        .points
        .iter()
        .step_by((contact.points.len() / 10).max(1))
        .filter_map(|(w, c)| state_at(host, level, *c).map(|s| (*w, s)))
        .collect()
}

/// Takes the block at `p` out of the level (water stays where it was waterlogged).
fn clear(
    host: &mut dyn ServerHost,
    level: LevelRef,
    p: Pos,
    id: BlockState,
    flags: BlockUpdateFlags,
) {
    let wet = prop(host, id, "waterlogged").as_deref() == Some("true");
    let Some(left) = host.block_state_id(if wet {
        "minecraft:water"
    } else {
        "minecraft:air"
    }) else {
        return;
    };
    let _ = level.set_block(host, p, left, flags);
}

/// An item pops out at `at`; or, for an `owner` in this level, goes into their
/// inventory (what doesn't fit drops from them) or straight to their feet.
fn drop_item(
    host: &mut dyn ServerHost,
    level: LevelRef,
    rng: &mut rules::Rng,
    at: [f64; 3],
    item: ItemStack,
    owner: Owner,
) {
    if let Some((o, bag)) = owner
        && let Some(p) = host.player(o)
        && p.level(host) == Some(level)
        && let Some(feet) = p.entity().position(host)
    {
        if bag && p.give_stack(host, item.clone()).is_ok() {
            return;
        }
        let _ = level.spawn_item_with_motion(host, feet[0], feet[1], feet[2], item, [0.0; 3], 0);
        return;
    }
    let at = free_spot(host, level, at);
    let v = [rng.spread(0.08), 0.12 + rng.unit() * 0.1, rng.spread(0.08)];
    let _ = level.spawn_item_with_motion(host, at[0], at[1], at[2], item, v, 10);
}

/// `at`, or just above it when it is inside a solid block.
fn free_spot(host: &dyn Host, level: LevelRef, at: [f64; 3]) -> [f64; 3] {
    let mut p = at;
    for _ in 0..3 {
        let cell = [
            p[0].floor() as i32,
            p[1].floor() as i32,
            p[2].floor() as i32,
        ];
        let solid = state_at(host, level, cell).is_some_and(|id| {
            !is_air(&name_of(host, id))
                && collision(host, level, id, cell).is_none_or(|s| !s.is_empty())
        });
        if !solid {
            return p;
        }
        p[1] = p[1].floor() + 1.2;
    }
    p
}

// ---------------------------------------------------------------------------------
// Replanting
// ---------------------------------------------------------------------------------

/// Whether the block under `p` takes `sapling` (dirt, grass, podzol, moss, mud...;
/// clay too for a propagule).
fn ground_ok(host: &dyn Host, level: LevelRef, p: Pos, sapling: &str) -> bool {
    let Some(below) = state_at(host, level, [p[0], p[1] - 1, p[2]]) else {
        return false;
    };
    let name = name_of(host, below);
    if sapling.ends_with("_mushroom") {
        // a mushroom keeps to mycelium, podzol and nylium (the tag's name in 26.3,
        // and its name before)
        return in_tag(
            host,
            below,
            "minecraft:overrides_mushroom_light_requirement",
        ) || in_tag(host, below, "minecraft:mushroom_grow_block");
    }
    let fungus = sapling.ends_with("_fungus")
        && (in_tag(host, below, "minecraft:nylium")
            || name == "minecraft:mycelium"
            || name == "minecraft:soul_soil");
    fungus
        || in_tag(host, below, "minecraft:dirt")
        || in_tag(host, below, "minecraft:supports_vegetation")
        || name == "minecraft:farmland"
        || (sapling.ends_with("propagule") && name == "minecraft:clay")
}

/// Whether `p` has room for `sapling`: air, or water for a propagule.
fn room_for(host: &dyn Host, level: LevelRef, p: Pos, sapling: &str) -> Option<bool> {
    let here = state_at(host, level, p)?;
    let name = name_of(host, here);
    if is_air(&name) {
        Some(false)
    } else if sapling.ends_with("propagule") && name == "minecraft:water" {
        Some(true)
    } else {
        None
    }
}

/// What to replant, where, and whether the saplings come out of the loot: the
/// tree's own sapling, on its places that stand on soil. With "une pousse du
/// butin" they are taken out of what the tree gives (as many as it gives, one a
/// place); without loot (creative, no block drops) they are given.
#[allow(clippy::too_many_arguments)]
fn plan_replant(
    host: &dyn Host,
    level: LevelRef,
    config: &Config,
    tree: &rules::Tree,
    cut_name: &str,
    leaf: Option<&str>,
    drops: bool,
    loot: &mut [Vec<ItemStack>],
    hung: &mut [(Pos, BlockState, Vec<ItemStack>)],
) -> Option<(&'static str, Vec<Pos>, bool)> {
    let mode = config.get(Key::Replant);
    if mode == 0 {
        return None;
    }
    let sapling = rules::sapling(species(cut_name), leaf)?;
    let mut spots: Vec<Pos> = tree
        .spots()
        .into_iter()
        .filter(|p| ground_ok(host, level, *p, sapling))
        .collect();
    // a fungus never drops its own: the wart's tree gives it back free
    let from_loot = mode == 1 && drops && !sapling.ends_with("_fungus");
    if from_loot {
        let key = Identifier::parse(sapling)
            .ok()
            .and_then(|id| host.item_key(&id));
        let mut have = 0;
        let lists = loot.iter_mut().chain(hung.iter_mut().map(|h| &mut h.2));
        for list in lists {
            list.retain_mut(|stack| {
                if have == spots.len() || Some(stack.item()) != key {
                    return true;
                }
                let take = (spots.len() - have).min(stack.count() as usize);
                have += take;
                let left = stack.count() - take as u32;
                left > 0 && stack.set_count(left).is_ok()
            });
        }
        spots.truncate(have);
    }
    (!spots.is_empty()).then_some((sapling, spots, from_loot))
}

/// Plants `sapling` at each of `spots` still free on soil, with a little pop and
/// green sparkles; one that came out of the loot and has no room drops there.
fn replant(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    rng: &mut rules::Rng,
    sapling: &'static str,
    spots: &[Pos],
    from_loot: bool,
) {
    let Some(plain) = host.block_state_id(sapling) else {
        eprintln!("Chop!: no block {sapling} to replant");
        return;
    };
    for p in spots {
        let at = [0, 1, 2].map(|k| f64::from(p[k]) + 0.5);
        let wet =
            room_for(host, level, *p, sapling).filter(|_| ground_ok(host, level, *p, sapling));
        let Some(wet) = wet else {
            if from_loot && let Some(item) = stack_of(host, sapling) {
                drop_item(host, level, rng, at, item, None);
            }
            continue;
        };
        let state = if wet {
            with_prop(host, plain, "waterlogged", "true").unwrap_or(plain)
        } else {
            plain
        };
        let _ = level.set_block(host, *p, state, normal());
        sound(
            host,
            level,
            config,
            "minecraft:block.grass.place",
            at,
            0.8,
            1.2,
        );
        sound(host, level, config, BUBBLE, at, 0.4, 1.6);
        particles(
            host,
            level,
            config,
            Fx::HappyVillager,
            None,
            at,
            [0.3, 0.3, 0.3],
            0.0,
            6.0,
        );
    }
}

// ---------------------------------------------------------------------------------
// A log being dug
// ---------------------------------------------------------------------------------

/// The logs being dug in the level `dim` tremble a tick: harder as the crack grows,
/// with a little knock, a few chips and leaves shivering off the crown. One that
/// broke waits a few ticks for its felling to take it; one let go stops.
fn tick_trembles(host: &mut dyn ServerHost, level: LevelRef, st: &mut State, dim: &str) {
    let config = st.config.clone();
    let mut i = 0;
    while i < st.trembles.len() {
        if st.trembles[i].dim != dim {
            i += 1;
            continue;
        }
        st.trembles[i].age += 1;
        let t = &st.trembles[i];
        let standing = state_at(host, level, t.pos) == Some(t.state);
        let done = if !standing {
            st.trembles[i].broken += 1;
            st.trembles[i].broken > 3
        } else if t.shaking {
            t.stopped || t.age > t.ticks
        } else {
            t.stopped
                || !config.on(Key::Shake)
                || (t.stage.is_none() && t.age > DIG_START)
                || t.age.saturating_sub(t.staged_at) > DIG_STALL
        };
        if done {
            let t = st.trembles.remove(i);
            if let Some(d) = t.display {
                remove(host, d);
            }
            continue;
        }
        if !standing {
            i += 1;
            continue;
        }
        if !st.trembles[i].checked {
            let t = &st.trembles[i];
            if !t.shaking {
                match t.first_stage {
                    // the dig hasn't shown yet
                    None => {
                        i += 1;
                        continue;
                    }
                    // it breaks in a tick or two: nothing to shake
                    Some(s) if s >= 5 => {
                        st.trembles.remove(i);
                        continue;
                    }
                    _ => {}
                }
            }
            match tremble_start(host, level, st, i) {
                Some(setup) => {
                    let t = &mut st.trembles[i];
                    t.checked = true;
                    t.display = Some(setup.display);
                    t.leaves = setup.leaves;
                    t.tint = setup.tint;
                    t.preview = setup.preview;
                }
                None => {
                    st.trembles.remove(i);
                    continue;
                }
            }
        }
        let t = &st.trembles[i];
        let strength = if t.shaking {
            0.6
        } else {
            ((f64::from(t.stage.unwrap_or(0)) + 1.0) / 10.0).min(1.0)
        };
        // where it will fall, for the players who want to see it
        if t.age % 4 == 1 && !t.preview.is_empty() {
            let dust = Fx::Dust([1.0, 0.85, 0.35], 0.9);
            let points = t.preview.clone();
            let world = config.on(Key::Preview);
            let want = move |u: u128| match perso_of(u).preview {
                -1 => world,
                v => v == 1,
            };
            for at in points {
                particles_to(
                    host,
                    level,
                    &config,
                    dust,
                    None,
                    at,
                    [0.15, 0.0, 0.15],
                    0.0,
                    1.0,
                    &want,
                );
            }
        }
        let t = &st.trembles[i];
        if let Some(d) = t.display {
            let (tr, scale) = rules::tremble_pose(t.age, strength, t.face);
            pose(host, d, tr, [0.0, 0.0, 0.0, 1.0], scale);
        }
        let t = &st.trembles[i];
        let (age, shaking, state, face_at) = (
            t.age,
            t.shaking,
            t.state,
            [0, 1, 2].map(|k| f64::from(t.pos[k]) + 0.5 + 0.55 * f64::from(t.face[k])),
        );
        if age % 2 == 0 {
            let pitch = if age % 4 == 0 { 1.5 } else { 1.7 } + 0.2 * strength;
            sound(
                host,
                level,
                &config,
                "minecraft:block.wood.hit",
                face_at,
                0.3,
                pitch,
            );
            block_bits(host, level, &config, state, face_at, 2.0);
        }
        let t = &st.trembles[i];
        if age % 3 == 0 && !t.leaves.is_empty() {
            let leaf = t.leaves[st.rng.below(t.leaves.len() as u32) as usize];
            let tint = t.tint;
            if let Some(id) = state_at(host, level, leaf) {
                let name = name_of(host, id);
                let at = [0, 1, 2].map(|k| f64::from(leaf[k]) + 0.5);
                if let Some(fx) = leaf_particle(&name, tint) {
                    particles(
                        host,
                        level,
                        &config,
                        fx,
                        None,
                        [at[0], at[1] - 0.6, at[2]],
                        [0.4, 0.1, 0.4],
                        0.0,
                        if shaking { 3.0 } else { 1.0 },
                    );
                }
                if age % 6 == 0 {
                    sound(host, level, &config, step_sound(&name), at, 0.3, 1.1);
                }
            }
        }
        i += 1;
    }
}

/// Trees shaken by hand in the level `dim`: the trunk shivers, leaves flutter
/// down and a few of what leaves give (sticks, saplings, apples) fall out; a
/// tree gives once every 30 seconds.
fn tick_shakes(host: &mut dyn ServerHost, level: LevelRef, st: &mut State, dim: &str) {
    if st.shakes.is_empty() {
        return;
    }
    let config = st.config.clone();
    let (here, rest): (Vec<ShakeReq>, Vec<ShakeReq>) = std::mem::take(&mut st.shakes)
        .into_iter()
        .partition(|r| r.dim == dim);
    st.shakes = rest;
    for r in here {
        // one shake a player every half second
        if st.shook.get(&r.player).is_some_and(|t| st.tick < t + 10) {
            continue;
        }
        st.shook.insert(r.player, st.tick);
        let Some((base, base_state)) = trunk_base(host, level, r.pos) else {
            continue;
        };
        let Some(state) = state_at(host, level, r.pos) else {
            continue;
        };
        let tree = {
            let placed = st.placed.get(dim);
            let protect = config.on(Key::Protect);
            let is_placed = |q: Pos| protect && placed.is_some_and(|set| set.contains(&q));
            let mut reader = Reader::new(host, level, &config);
            let wood = reader.wood(&name_of(host, base_state));
            let mut look = rules::Look::new(&mut reader);
            rules::scan(
                &mut look,
                &is_placed,
                base,
                wood,
                &rules::Limits::of(&config),
            )
        };
        let Ok(tree) = tree else {
            continue;
        };
        // the trunk shivers a moment
        st.trembles.push(Tremble {
            dim: r.dim.clone(),
            pos: r.pos,
            face: r.face,
            state,
            player: r.player,
            ticks: 8,
            age: 0,
            display: None,
            leaves: Vec::new(),
            checked: false,
            stopped: false,
            broken: 0,
            shaking: true,
            tint: None,
            preview: Vec::new(),
            stage: None,
            first_stage: None,
            staged_at: 0,
        });
        let leaf_name = tree
            .leaves
            .first()
            .and_then(|p| state_at(host, level, *p))
            .map(|id| name_of(host, id));
        if let Some(l) = &leaf_name {
            for k in 0..2.min(tree.leaves.len()) {
                let p = tree.leaves[(k * 7) % tree.leaves.len()];
                let at = [0, 1, 2].map(|i| f64::from(p[i]) + 0.5);
                sound(
                    host,
                    level,
                    &config,
                    step_sound(l),
                    at,
                    0.7,
                    0.9 + 0.2 * k as f64,
                );
            }
        }
        let key = (r.dim.clone(), base);
        if st.shaken.get(&key).is_some_and(|t| st.tick < t + 600) {
            continue;
        }
        st.shaken.insert(key, st.tick);
        let rolls = (tree.leaves.len() / 4).clamp(2, 12);
        for _ in 0..rolls {
            let leaf = tree.leaves[st.rng.below(tree.leaves.len() as u32) as usize];
            let Some(id) = state_at(host, level, leaf) else {
                continue;
            };
            if !in_tag(host, id, "minecraft:leaves") {
                continue;
            }
            let at = [0, 1, 2].map(|i| f64::from(leaf[i]) + 0.5 - if i == 1 { 0.6 } else { 0.0 });
            for item in loot_at(host, level, leaf, None) {
                if item.item().id(host).ends_with("_leaves") {
                    continue;
                }
                drop_item(host, level, &mut st.rng, at, item, None);
            }
        }
    }
}

/// What a log that starts to tremble shows.
struct TrembleSetup {
    display: DisplayRef,
    leaves: Vec<Pos>,
    tint: Option<u32>,
    preview: Vec<[f64; 3]>,
}

/// A dig that just started (or a tree shaken): if its digger may fell the tree
/// behind the log, the log's trembling display, some of the tree's lowest leaves,
/// their tint and, for a dig, where the trunk will lie.
fn tremble_start(
    host: &mut dyn ServerHost,
    level: LevelRef,
    st: &State,
    i: usize,
) -> Option<TrembleSetup> {
    let t = &st.trembles[i];
    let config = st.config.clone();
    let who = cutter(host, t.player)?;
    if who.dim != t.dim || (!t.shaking && who.game_mode == 1) {
        return None;
    }
    if !t.shaking {
        let axe = who
            .held
            .as_ref()
            .is_some_and(|s| item_in_tag(host, s, "minecraft:axes"));
        if !rules::fells(&config, who.game_mode, who.sneaking, axe) {
            return None;
        }
    }
    let (tree, preview) = {
        let placed = st.placed.get(&t.dim);
        let protect = config.on(Key::Protect);
        let is_placed = |q: Pos| protect && placed.is_some_and(|set| set.contains(&q));
        let mut reader = Reader::new(host, level, &config);
        let wood = reader.wood(&name_of(host, t.state));
        let mut look = rules::Look::new(&mut reader);
        let tree = rules::scan(
            &mut look,
            &is_placed,
            t.pos,
            wood,
            &rules::Limits::of(&config),
        )
        .ok()?;
        drop(look);
        // where the trunk will lie, as it would fall if the log broke now (the
        // log's own place counted open)
        let mut preview = Vec::new();
        if !t.shaking && config.on(Key::Preview) && config.on(Key::Animation) {
            let mut holed = Holed {
                inner: &mut reader,
                hole: t.pos,
            };
            let mut look = rules::Look::new(&mut holed);
            let sink = rules::gap(&mut look, &tree);
            let dir = rules::choose_dir(
                &tree,
                &mut look,
                f64::from(who.yaw),
                config.on(Key::Aim),
                config.get(Key::Cone),
                sink,
            );
            let body = rules::Body::new(&tree, dir, sink);
            let stop = body.contact(&mut look, true).theta;
            let mut seen = HashSet::new();
            for (k, _) in body.parts.iter().enumerate().filter(|(_, p)| !p.leaf) {
                let c = body.center(k, stop);
                let cell = [c[0].floor() as i32, c[2].floor() as i32];
                if seen.insert(cell)
                    && let Some(y) =
                        ground_under(host, level, [cell[0], c[1].floor() as i32 + 1, cell[1]])
                {
                    preview.push([c[0], y, c[2]]);
                }
            }
        }
        (tree, preview)
    };
    let mut leaves = tree.leaves.clone();
    leaves.sort_by_key(|p| p[1]);
    leaves.truncate(12);
    let leaf = tree
        .leaves
        .first()
        .and_then(|p| state_at(host, level, *p))
        .map(|id| name_of(host, id));
    let tint = tint_for(host, level, leaf.as_deref(), t.pos);
    // the display stands in the cell in front of the face dug, so it is lit as
    // that side is
    let anchor = [0, 1, 2].map(|k| f64::from(t.pos[k] + t.face[k]) + 0.5);
    let (tr, scale) = rules::tremble_pose(0, 0.0, t.face);
    let display = summon_display(host, &t.dim, anchor, t.state, tr, scale, st.session, 0)?;
    Some(TrembleSetup {
        display,
        leaves,
        tint,
        preview,
    })
}

/// The level with one cell open: the log being dug, as it will be once broken.
struct Holed<'r, 'g> {
    inner: &'r mut Reader<'g>,
    hole: Pos,
}

impl rules::World for Holed<'_, '_> {
    fn cell(&mut self, p: Pos) -> Option<Cell> {
        if p == self.hole {
            return Some(Cell::Open);
        }
        rules::World::cell(self.inner, p)
    }
}

/// Whether the cell at `p` is open (air, plants).
fn look_is_open(host: &dyn Host, level: LevelRef, p: Pos) -> bool {
    state_at(host, level, p).is_some_and(|id| {
        is_air(&name_of(host, id))
            || host.block_states().replaceable(id) == Some(true)
            || collision(host, level, id, p).is_some_and(|s| s.is_empty())
    })
}

/// The top of the ground under `p` (searching six blocks down): just above the
/// first block something can stand on.
fn ground_under(host: &dyn Host, level: LevelRef, p: Pos) -> Option<f64> {
    for dy in 0..8 {
        let q = [p[0], p[1] - dy, p[2]];
        if !look_is_open(host, level, q) && state_at(host, level, q).is_some() {
            return Some(f64::from(q[1]) + 1.05);
        }
    }
    None
}

// ---------------------------------------------------------------------------------
// A chain reaction
// ---------------------------------------------------------------------------------

/// A falling trunk came down on other trees' logs.
struct Knock {
    hitter: u64,
    credit: Option<(u128, String)>,
    cells: Vec<Pos>,
    dir: [f64; 2],
    generation: u32,
    tool: Option<ItemStack>,
    drops: bool,
    owner: Owner,
    omega: f64,
    at: [f64; 3],
}

/// The lowest log of the trunk `cell` is in (same kind of log all the way down).
fn trunk_base(host: &dyn Host, level: LevelRef, cell: Pos) -> Option<(Pos, BlockState)> {
    let id = state_at(host, level, cell)?;
    if !is_trunk(host, id) {
        return None;
    }
    let kind = species(&name_of(host, id)).to_owned();
    let mut base = (cell, id);
    for _ in 0..64 {
        let below = [base.0[0], base.0[1] - 1, base.0[2]];
        match state_at(host, level, below) {
            Some(b) if is_trunk(host, b) && species(&name_of(host, b)) == kind => {
                base = (below, b);
            }
            _ => break,
        }
    }
    Some(base)
}

/// Each tree whose trunk was hit falls in turn, the way it was pushed; the tree
/// that hit it, its way clear, falls on.
fn chain(host: &mut dyn ServerHost, level: LevelRef, st: &mut State, dim: &str, k: Knock) {
    let config = st.config.clone();
    let mut bases = HashSet::new();
    let mut felled = false;
    for cell in &k.cells {
        let Some((base, state)) = trunk_base(host, level, *cell) else {
            continue;
        };
        if !bases.insert(base) {
            continue;
        }
        let start = Start {
            dim: dim.to_owned(),
            cut: base,
            cut_state: state,
            yaw: rules::dir_yaw(k.dir),
            aim: true,
            cone: 20,
            tool: k.tool.clone(),
            drops: k.drops,
            owner: k.owner,
            cutter: None,
            credit: k.credit.clone(),
            wear: false,
            tired: false,
            ghost: None,
            generation: k.generation,
            standing: true,
        };
        if fell_tree(host, level, st, start) {
            felled = true;
        }
    }
    if !felled {
        return;
    }
    // bonk!
    sound(
        host,
        level,
        &config,
        "minecraft:block.wood.hit",
        k.at,
        1.0,
        0.7,
    );
    sound(host, level, &config, CRACK, k.at, 0.7, 0.8);
    sound(host, level, &config, PLOP, k.at, 0.7, 0.7);
    particles(
        host,
        level,
        &config,
        Fx::Crit,
        None,
        k.at,
        [0.3, 0.3, 0.3],
        0.3,
        8.0,
    );
    let Some(t) = st.trees.iter_mut().find(|t| t.id == k.hitter) else {
        return;
    };
    let (contact, crush, hits) = {
        let mut reader = Reader::new(host, level, &config);
        let mut look = rules::Look::new(&mut reader);
        let contact = t.fell.body.contact(&mut look, true);
        let crush = t.fell.body.crush(&mut look, contact.theta);
        let hits = log_hits(&mut look, &contact);
        (contact, crush, hits)
    };
    t.contacts = dust_points(host, level, &contact);
    t.hits = hits;
    t.resumes += 1;
    t.settled = false;
    t.fell.resume(&contact, crush, k.omega * 0.55);
}

// ---------------------------------------------------------------------------------
// A tick of a falling tree
// ---------------------------------------------------------------------------------

/// Moves `t` a tick in its level (in place): whether it is done, and the trees its
/// trunk knocks into.
fn tick_tree(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    rng: &mut rules::Rng,
    t: &mut Falling,
) -> (bool, Option<Knock>) {
    let was_falling = t.fell.phase == Phase::Falling;
    let frame = t.fell.tick();
    let theta = frame.theta;
    play_later(host, level, config, t);

    // the log that trembled as it was dug pops
    if let Some(id) = t.ghost.take() {
        remove(host, id);
        let at = t.anchor;
        block_bits(host, level, config, t.cut_state, at, 10.0);
        particles(
            host,
            level,
            config,
            Fx::Poof,
            None,
            at,
            [0.2, 0.2, 0.2],
            0.04,
            5.0,
        );
        sound(host, level, config, PLOP, at, 0.8, 1.3);
        sound(host, level, config, BUBBLE, at, 0.7, 1.2);
        sound(host, level, config, CRACK, at, 0.5, 1.2);
    }

    if frame.moved {
        let q = t.fell.body.quat(theta);
        let remaining: Vec<usize> = t.fell.remaining().collect();
        for i in remaining {
            if let Some(d) = t.ents[i] {
                let tr = t
                    .fell
                    .body
                    .translation(t.fell.body.parts[i].cell, theta, t.anchor);
                pose(host, d, tr, q, 1.0);
            }
        }
    }
    if frame.landed.is_some() {
        // down on the stump: pomf
        let body = &t.fell.body;
        let at = [body.pivot[0], body.pivot[1] - body.sink, body.pivot[2]];
        sound(host, level, config, WOOL, at, 0.9, 0.75);
        sound(host, level, config, THUD, at, 0.5, 0.8);
        block_bits(host, level, config, t.cut_state, at, 6.0);
    }
    if !t.creaked && t.fell.phase == Phase::Falling {
        // it starts to go: the fibres crack and the trunk groans
        t.creaked = true;
        let body = &t.fell.body;
        let base = [
            body.pivot[0],
            body.pivot[1] - body.lowered + 0.5,
            body.pivot[2],
        ];
        let mid = body.center(t.middle, theta);
        sound(host, level, config, CRACK, base, 0.7, 0.65);
        sound(host, level, config, CREAK, mid, 0.7, 0.55);
        let now = t.fell.age;
        t.later.push((now + 7, CREAK, mid, 0.5, 0.5));
        t.creak_age = Some(now);
        // a big tree's birds take off
        if config.on(Key::Birds) && t.leaves.len() >= 25 {
            let crown = t.fell.body.center(t.tip, theta);
            sound(
                host,
                level,
                config,
                "minecraft:entity.parrot.fly",
                crown,
                0.7,
                1.1,
            );
            sound(
                host,
                level,
                config,
                "minecraft:entity.parrot.ambient",
                crown,
                0.6,
                1.5,
            );
            t.later
                .push((now + 3, "minecraft:entity.parrot.ambient", crown, 0.5, 1.75));
            t.later
                .push((now + 5, "minecraft:entity.parrot.fly", crown, 0.5, 1.3));
            particles(
                host,
                level,
                config,
                Fx::Cloud,
                None,
                [crown[0], crown[1] + 1.0, crown[2]],
                [1.2, 0.4, 1.2],
                0.06,
                6.0,
            );
        }
    }
    // apples come loose as it falls; what is left goes when it lands
    if let Some(start) = t.creak_age
        && !t.apples.is_empty()
    {
        let lying = t.fell.phase != Phase::Falling;
        let now = t.fell.age;
        let mut k = 0;
        let mut plopped = false;
        while k < t.apples.len() {
            if lying || now >= start + t.apples[k].2 {
                let (i, apple, _) = t.apples.remove(k);
                let at = t.fell.body.center(i, theta);
                drop_item(host, level, rng, at, apple, t.owner);
                if !plopped {
                    plopped = true;
                    sound(host, level, config, PLOP, at, 0.3, 1.5);
                }
            } else {
                k += 1;
            }
        }
    }
    if was_falling {
        falling_effects(host, level, config, t);
        strike(host, level, config, t);
    }
    let mut knock = None;
    if let Some(impact) = frame.impact {
        impact_effects(host, level, config, rng, t, impact);
        let tip_speed = impact.omega.abs() * t.fell.body.reach;
        if impact.first
            && impact.ground
            && i64::from(t.generation) < i64::from(config.get(Key::Chain))
            && t.resumes < 3
            && !t.hits.is_empty()
            && tip_speed > 2.0
        {
            let at = t.contacts.first().map_or(t.anchor, |c| c.0);
            knock = Some(Knock {
                hitter: t.id,
                credit: t.credit.clone(),
                cells: std::mem::take(&mut t.hits),
                dir: t.fell.body.dir,
                generation: t.generation + 1,
                tool: t.tool.clone(),
                drops: t.drops,
                owner: t.owner,
                omega: impact.omega,
                at,
            });
        }
    }
    if was_falling && t.fell.phase != Phase::Falling && !t.settled {
        // its displays hold their last pose: the server keeps it, so players who
        // come later see it lying where it lies
        t.settled = true;
    }

    // leaves crushed under the trunk
    let mut squished = false;
    for i in frame.crushed {
        let at = t.fell.body.center(i, theta);
        if let Some(d) = t.ents[i].take() {
            remove(host, d);
        }
        block_bits(host, level, config, t.states[i], at, 4.0);
        if !squished {
            squished = true;
            sound(
                host,
                level,
                config,
                "minecraft:block.moss_carpet.place",
                at,
                0.5,
                1.2,
            );
            if let Some(l) = &t.leaf {
                sound(host, level, config, break_sound(l), at, 0.4, 1.3);
            }
        }
        for item in std::mem::take(&mut t.loot[i]) {
            drop_item(host, level, rng, at, item, t.owner);
        }
    }

    if matches!(t.fell.phase, Phase::Lying(_) | Phase::Breaking)
        && config.on(Key::Leaves)
        && t.fell.age.is_multiple_of(3)
    {
        twirl(host, level, config, rng, t);
    }

    // from the base up, each block puffs up and pops
    for &(i, scale, lift) in &frame.popping {
        if let Some(d) = t.ents[i] {
            let (tr, q) = t.fell.body.pose(i, theta, t.anchor, scale, lift);
            pose(host, d, tr, q, scale as f32);
        }
    }
    if let Some(n) = frame.note
        && let Some(&(i, ..)) = frame.popping.last()
    {
        // the pops rise in pitch as the breaking climbs
        let at = t.fell.body.center(i, theta);
        sound(host, level, config, BUBBLE, at, 0.5, 0.75 + 0.07 * n as f64);
        particles(
            host,
            level,
            config,
            Fx::HappyVillager,
            None,
            [at[0], at[1] + 0.5, at[2]],
            [0.3, 0.2, 0.3],
            0.0,
            2.0,
        );
    }
    let mut plops = 0;
    for i in frame.broke {
        let at = t.fell.body.center(i, theta);
        if let Some(d) = t.ents[i].take() {
            remove(host, d);
        }
        block_bits(host, level, config, t.states[i], at, 3.0);
        particles(
            host,
            level,
            config,
            Fx::Poof,
            None,
            at,
            [0.15, 0.15, 0.15],
            0.02,
            1.0,
        );
        if plops < 1 {
            plops += 1;
            sound(host, level, config, PLOP, at, 0.3, 1.2 + rng.unit() * 0.4);
        }
        for item in std::mem::take(&mut t.loot[i]) {
            drop_item(host, level, rng, at, item, t.owner);
        }
    }
    if frame.done {
        for (i, apple, _) in std::mem::take(&mut t.apples) {
            let at = t.fell.body.center(i, theta);
            drop_item(host, level, rng, at, apple, t.owner);
        }
    }
    if frame.done
        && let Some((sapling, spots, from_loot)) = t.replant.take()
    {
        replant(host, level, config, rng, sapling, &spots, from_loot);
    }
    if frame.done {
        // all gone: a last pop and a few hearts where the crown was
        let at = t.fell.body.center(t.tip, theta);
        sound(host, level, config, PLOP, at, 0.7, 1.0);
        sound(host, level, config, BUBBLE, at, 0.6, 1.5);
        sound(
            host,
            level,
            config,
            "minecraft:entity.item.pickup",
            at,
            0.4,
            1.6,
        );
        particles(
            host,
            level,
            config,
            Fx::Heart,
            None,
            [at[0], at[1] + 0.5, at[2]],
            [0.6, 0.3, 0.6],
            0.0,
            3.0,
        );
    }
    (frame.done, knock)
}

/// The sounds whose tick has come.
fn play_later(host: &mut dyn ServerHost, level: LevelRef, config: &Config, t: &mut Falling) {
    let now = t.fell.age;
    let mut i = 0;
    while i < t.later.len() {
        if t.later[i].0 <= now {
            let (_, name, at, volume, pitch) = t.later.remove(i);
            sound(host, level, config, name, at, volume, pitch);
        } else {
            i += 1;
        }
    }
}

/// The crown rustles as it gathers speed, and flutters once it is fast.
fn falling_effects(host: &mut dyn ServerHost, level: LevelRef, config: &Config, t: &mut Falling) {
    let body = &t.fell.body;
    let tip_speed = t.fell.motion.omega.abs() * body.reach;
    let crown = body.center(t.tip, t.fell.theta());
    if let Some(leaf) = &t.leaf
        && tip_speed > 3.0
        && t.fell.age.is_multiple_of(4)
    {
        let v = (tip_speed / 14.0).clamp(0.2, 0.7);
        sound(host, level, config, step_sound(leaf), crown, v, 0.8);
    }
    if !t.whoosh && tip_speed > 6.0 {
        t.whoosh = true;
        sound(
            host,
            level,
            config,
            "minecraft:entity.parrot.fly",
            crown,
            0.7,
            0.75,
        );
        sound(
            host,
            level,
            config,
            "minecraft:entity.player.attack.sweep",
            crown,
            0.2,
            0.8,
        );
    }
}

/// The ground takes it softly: a deep pomf and a squish, then a little rising
/// arpeggio; the soil's dust and a burst of leaves. A bounce is a boing.
fn impact_effects(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    rng: &mut rules::Rng,
    t: &mut Falling,
    impact: rules::Impact,
) {
    if !impact.ground {
        return;
    }
    let theta = t.fell.theta();
    let body = &t.fell.body;
    let logs = body.logs as f64;
    let speed = (impact.omega.abs() * body.reach / 10.0).clamp(0.4, 1.3);
    let mid = body.center(t.middle, theta);
    let crown = body.center(t.tip, theta);
    if impact.first {
        // a deep wooden thud as big as the tree, softened, branches snapping and
        // the crown crashing, then a few bubbly pops
        let deep = (0.75 - logs * 0.008).clamp(0.5, 0.75);
        sound(host, level, config, THUD, mid, 1.0 * speed, deep);
        sound(host, level, config, WOOL, mid, 1.0 * speed, 0.6);
        sound(host, level, config, CRACK, mid, 0.7, 0.6);
        if let Some(l) = &t.leaf {
            sound(host, level, config, break_sound(l), crown, 0.9, 0.8);
        }
        let now = t.fell.age;
        for (k, pitch) in [(2, 1.0), (3, 1.25), (5, 1.45)] {
            t.later.push((now + k, BUBBLE, crown, 0.4, pitch));
        }
    } else {
        // a bounce: a lighter thud and a little pop
        sound(host, level, config, THUD, mid, 0.4 + 0.3 * speed, 1.0);
        sound(
            host,
            level,
            config,
            "minecraft:block.wool.step",
            mid,
            0.6,
            0.9,
        );
        sound(host, level, config, BUBBLE, mid, 0.3, 1.3);
    }
    if config.on(Key::Dust) {
        let k = if impact.first { 1.0 } else { 0.35 };
        for (at, ground) in t.contacts.clone() {
            particles(
                host,
                level,
                config,
                Fx::DustPillar,
                Some(ground),
                at,
                [0.3, 0.05, 0.3],
                0.2,
                5.0 * k * speed,
            );
            particles(
                host,
                level,
                config,
                Fx::Block,
                Some(ground),
                at,
                [0.45, 0.1, 0.45],
                0.15,
                8.0 * k * speed,
            );
            if impact.first {
                particles(
                    host,
                    level,
                    config,
                    Fx::Poof,
                    None,
                    at,
                    [0.3, 0.1, 0.3],
                    0.02,
                    1.0,
                );
            }
        }
    }
    if impact.first
        && config.on(Key::Leaves)
        && let Some(l) = t.leaf.clone()
    {
        let alive: Vec<usize> = t
            .leaves
            .iter()
            .copied()
            .filter(|i| t.fell.alive[*i])
            .collect();
        for _ in 0..10.min(alive.len()) {
            let i = alive[rng.below(alive.len() as u32) as usize];
            let at = t.fell.body.center(i, theta);
            if let Some(fx) = leaf_particle(&l, t.tint) {
                particles(
                    host,
                    level,
                    config,
                    fx,
                    None,
                    at,
                    [0.5, 0.4, 0.5],
                    0.03,
                    3.0,
                );
            }
        }
    }
}

/// Leaves drift down from the crown while it lies there.
fn twirl(
    host: &mut dyn ServerHost,
    level: LevelRef,
    config: &Config,
    rng: &mut rules::Rng,
    t: &Falling,
) {
    let Some(l) = &t.leaf else {
        return;
    };
    let alive: Vec<usize> = t
        .leaves
        .iter()
        .copied()
        .filter(|i| t.fell.alive[*i])
        .collect();
    if alive.is_empty() {
        return;
    }
    for _ in 0..3 {
        let i = alive[rng.below(alive.len() as u32) as usize];
        let mut at = t.fell.body.center(i, t.fell.theta());
        at[1] += 0.4;
        if let Some(fx) = leaf_particle(l, t.tint) {
            particles(host, level, config, fx, None, at, [0.4, 0.2, 0.4], 0.0, 1.0);
        }
    }
}

/// What the falling tree comes down on: mobs and players allowed by the settings,
/// hurt by how fast the block that meets them moves, and pushed along the fall.
fn strike(host: &mut dyn ServerHost, level: LevelRef, config: &Config, t: &mut Falling) {
    let omega = t.fell.motion.omega;
    if !config.on(Key::Damage) || omega.abs() < 0.3 {
        return;
    }
    let theta = t.fell.theta();
    let body = &t.fell.body;
    let reach = body.reach + 2.0;
    let pivot = body.pivot;
    let dir = body.dir;
    let near = |p: [f64; 3]| {
        let d: f64 = (0..3).map(|k| (p[k] - pivot[k]).powi(2)).sum();
        d <= reach * reach
    };
    let max = f64::from(config.get(Key::Strength));
    let harm = |hit: Option<(f64, bool)>| -> Option<f32> {
        let (speed, leaf) = hit?;
        let felt = if leaf { speed * 0.3 } else { speed };
        (felt >= 2.0).then(|| (felt * 0.9).clamp(1.0, max) as f32)
    };
    let lo = [pivot[0] - reach, pivot[1] - reach, pivot[2] - reach];
    let hi = [pivot[0] + reach, pivot[1] + reach, pivot[2] + reach];
    // mobs
    let mut hits = Vec::new();
    for e in level.entities_in(host, lo, hi).unwrap_or_default() {
        if t.struck.contains(&e.id()) || host.mob(e).is_none() {
            continue;
        }
        let Some(living) = host.living(e) else {
            continue;
        };
        if living.alive(host) != Some(true) {
            continue;
        }
        let Some(pos) = e.position(host) else {
            continue;
        };
        if !near(pos) {
            continue;
        }
        let kind = e.kind(host).map(|k| k.id(host)).unwrap_or_default();
        let ally = living.owner(host).is_some() || ALLIES.contains(&kind.as_str());
        let allowed = if ally {
            config.on(Key::Allies)
        } else if matches!(e.classification(host), Some(MobCategory::Monster)) {
            config.on(Key::Monsters)
        } else {
            config.on(Key::Animals)
        };
        if !allowed {
            continue;
        }
        let Some([lo, hi]) = e.bounding_box(host) else {
            continue;
        };
        if let Some(dmg) = harm(body.strike(theta, omega, lo, hi, &t.fell.alive)) {
            let resist = living
                .attribute_value(host, "minecraft:knockback_resistance")
                .unwrap_or(0.0)
                .clamp(0.0, 1.0);
            hits.push((e, dmg, 0.4 * (1.0 - resist)));
        }
    }
    for (e, dmg, strength) in hits {
        t.struck.insert(e.id());
        let before = e.velocity(host);
        let on_ground = e.on_ground(host).unwrap_or(false);
        // hurt by a falling block, then knocked back the way the tree falls: vanilla's
        // knockback (`LivingEntity.knockback`) from a hit just behind it, which
        // replaces whatever way the hurt itself pushed it
        if e.hurt(host, "minecraft:falling_block", dmg, None, None) == Some(true)
            && strength > 0.0
            && let Some(v) = before
        {
            let len = dir[0].hypot(dir[1]).max(1e-9);
            let k = [dir[0] / len * strength, dir[1] / len * strength];
            let up = if on_ground {
                (v[1] / 2.0 + strength).min(0.4)
            } else {
                v[1]
            };
            e.set_velocity(host, [v[0] / 2.0 + k[0], up, v[2] / 2.0 + k[1]]);
        }
    }
    // players
    if !config.on(Key::Players) {
        return;
    }
    let mut struck = Vec::new();
    for p in level.online_players(host).unwrap_or_default() {
        if t.struck_players.contains(&p.uuid())
            || !matches!(
                p.game_mode(host),
                Some(GameMode::Survival | GameMode::Adventure)
            )
            || !host
                .living(p.entity())
                .and_then(|l| l.health(host))
                .is_some_and(|h| h > 0.0)
        {
            continue;
        }
        let Some(pos) = p.entity().position(host) else {
            continue;
        };
        if !near(pos) {
            continue;
        }
        let [lo, hi] = p.entity().bounding_box(host).unwrap_or([
            [pos[0] - 0.3, pos[1], pos[2] - 0.3],
            [pos[0] + 0.3, pos[1] + 1.8, pos[2] + 0.3],
        ]);
        if let Some(dmg) = harm(body.strike(theta, omega, lo, hi, &t.fell.alive))
            && let Some(name) = p.name(host)
        {
            struck.push((p.uuid(), name, dmg, pos));
        }
    }
    for (uuid, name, dmg, pos) in struck {
        t.struck_players.insert(uuid);
        host.commands().run_command(&format!(
            "damage {name} {dmg:.1} minecraft:falling_block at {} {} {}",
            pos[0] - dir[0],
            pos[1],
            pos[2] - dir[1]
        ));
    }
}

/// `{"Item": ..., "PickupDelay": 10s}` as SNBT, for `summon minecraft:item`.
fn snbt(tag: &Tag) -> String {
    fn quoted(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for c in s.chars() {
            if c == '"' || c == '\\' {
                out.push('\\');
            }
            out.push(c);
        }
        out.push('"');
        out
    }
    match tag {
        Tag::End => String::new(),
        Tag::Byte(v) => format!("{v}b"),
        Tag::Short(v) => format!("{v}s"),
        Tag::Int(v) => v.to_string(),
        Tag::Long(v) => format!("{v}L"),
        Tag::Float(v) => format!("{:?}f", f32::from_bits(*v)),
        Tag::Double(v) => format!("{:?}d", f64::from_bits(*v)),
        Tag::String(s) => quoted(&s.to_string_lossy()),
        Tag::ByteArray(v) => format!(
            "[B;{}]",
            v.iter()
                .map(|b| format!("{}b", *b as i8))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Tag::IntArray(v) => format!(
            "[I;{}]",
            v.iter().map(i32::to_string).collect::<Vec<_>>().join(",")
        ),
        Tag::LongArray(v) => format!(
            "[L;{}]",
            v.iter()
                .map(|l| format!("{l}L"))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Tag::List(items) => format!("[{}]", items.iter().map(snbt).collect::<Vec<_>>().join(",")),
        Tag::Compound(map) => format!(
            "{{{}}}",
            map.iter()
                .map(|(k, v)| format!("{}:{}", quoted(&k.to_string_lossy()), snbt(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

/// Ends a tree at once: its displays go and what it still holds drops where each
/// block is. In the level in place it is done directly; elsewhere with commands.
fn end_tree(host: &mut dyn ServerHost, t: &mut Falling) {
    let theta = t.fell.theta();
    let level = host.level(&t.dim).filter(|l| l.in_place(host));
    let mut rng = rules::Rng::new(t.id ^ 0x5EED);
    if let Some(id) = t.ghost.take()
        && level.is_some()
    {
        remove(host, id);
    }
    for (i, apple, _) in std::mem::take(&mut t.apples) {
        t.loot[i].push(apple);
        t.fell.alive[i] = true;
    }
    let remaining: Vec<usize> = t.fell.remaining().collect();
    for i in remaining {
        t.fell.alive[i] = false;
        let at = t.fell.body.center(i, theta);
        let items = std::mem::take(&mut t.loot[i]);
        if let Some(level) = level {
            if let Some(d) = t.ents[i].take() {
                remove(host, d);
            }
            for item in items {
                drop_item(host, level, &mut rng, at, item, None);
            }
        } else {
            for item in items {
                let Some(nbt) = item.to_nbt(host) else {
                    continue;
                };
                let mut c = Compound::new();
                c.insert("Item".into(), Tag::Compound(nbt));
                c.insert("PickupDelay".into(), Tag::Short(10));
                let snbt = snbt(&Tag::Compound(c));
                host.commands().run_command(&format!(
                    "execute in {} run summon minecraft:item {} {} {} {snbt}",
                    t.dim, at[0], at[1], at[2]
                ));
            }
        }
    }
    if level.is_none() {
        host.commands().run_command(&format!(
            "execute in {} run kill @e[type=minecraft:block_display,tag=chop.t{}]",
            t.dim, t.id
        ));
    }
    if let Some((sapling, spots, from_loot)) = t.replant.take() {
        if let Some(level) = level {
            let config = Config::default();
            replant(host, level, &config, &mut rng, sapling, &spots, from_loot);
        } else {
            for p in spots {
                host.commands().run_command(&format!(
                    "execute in {} run setblock {} {} {} {sapling} keep",
                    t.dim, p[0], p[1], p[2]
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------------
// /chop
// ---------------------------------------------------------------------------------

fn style(color: &str) -> TextStyle {
    let mut s = TextStyle::default();
    s.color = Some(color.to_owned());
    s
}

fn span(text: &str, color: &str) -> Text {
    Text::literal(text).with_style(style(color))
}

fn bold(text: &str, color: &str) -> Text {
    let mut s = style(color);
    s.bold = Some(true);
    Text::literal(text).with_style(s)
}

/// `label` that runs `command` when clicked, with `hint` on hover.
fn button(label: &str, color: &str, hint: &str, command: &str) -> Text {
    let mut s = style(color);
    s.click = Some(TextClick::RunCommand(command.to_owned()));
    s.hover = Some(TextHover::Text(Box::new(Text::literal(hint))));
    Text::literal(label).with_style(s)
}

fn hinted(text: &str, color: &str, hint: &str) -> Text {
    let mut s = style(color);
    s.hover = Some(TextHover::Text(Box::new(Text::literal(hint))));
    Text::literal(text).with_style(s)
}

/// `/chop` in every player's command tree. The settings are checked when run, so
/// the owner of a single player world uses them without cheats.
pub(crate) fn commands(commands: &mut Vec<Command>) {
    for usage in [
        "chop",
        "chop on",
        "chop off",
        "chop aide",
        "chop config",
        "chop config <page:word>",
        "chop set <option:word> <value:integer>",
        "chop reset",
        "chop perso",
        "chop perso <option:word> <value:integer>",
        "chop stats",
    ] {
        commands.push(Command::new(usage));
    }
}

/// `run_mod_command`: `/chop ...`. Whether it was ours.
pub(crate) fn perform(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    command: &CommandInvocation,
) -> bool {
    if command.literals.first().map(String::as_str) != Some("chop") {
        return false;
    }
    let CommandSource::Player {
        player, permission, ..
    } = source
    else {
        let _ = host
            .commands()
            .feedback(span("🪓 Chop! se règle en jeu, par un joueur.", "red"));
        return true;
    };
    // the words as typed: the literals, then the arguments in their order
    let mut typed = command.literals.clone();
    for argument in &command.arguments {
        match &argument.value {
            CommandArgumentValue::String(s) => typed.push(s.clone()),
            CommandArgumentValue::Integer(v) => typed.push(v.to_string()),
            _ => {}
        }
    }
    let words: Vec<&str> = typed.iter().map(String::as_str).collect();
    let uuid = player.uuid();
    let admin = permission.level() >= 2 || owner(host, uuid);
    let reply = with_state(host, |_, st| -> Vec<Text> {
        match words[1..] {
            [] | ["on"] | ["off"] => {
                let off = match words.get(1) {
                    Some(&"on") => false,
                    Some(&"off") => true,
                    _ => !st.off.contains(&uuid),
                };
                if off {
                    st.off.insert(uuid);
                } else {
                    st.off.remove(&uuid);
                }
                st.dirty = true;
                vec![toggled(off)]
            }
            ["config" | "set" | "reset", ..] if !admin => vec![span(
                "🪓 Chop! Les réglages sont réservés aux opérateurs (et au propriétaire du monde).",
                "red",
            )],
            ["config"] => page(&st.config, Section::Fall),
            ["config", w] => match Section::from_word(w) {
                Some(s) => page(&st.config, s),
                None => vec![span(
                    "🪓 Chop! Pages : chute, effets, degats, regles.",
                    "red",
                )],
            },
            ["set", word, value] => match value.parse::<i32>() {
                Ok(v) => match st.config.set(word, v) {
                    Ok(o) => {
                        st.dirty = true;
                        page(&st.config, o.section)
                    }
                    Err(e) => vec![span(&format!("🪓 Chop! {e}."), "red")],
                },
                Err(_) => vec![span(
                    "🪓 Chop! La valeur doit être un nombre entier.",
                    "red",
                )],
            },
            ["reset"] => {
                st.config = Config::default();
                st.dirty = true;
                let mut lines = vec![span("🪓 Chop! Réglages par défaut remis.", "green")];
                lines.extend(page(&st.config, Section::Fall));
                lines
            }
            ["perso"] => perso_page(&st.config, st.perso.get(&uuid).copied().unwrap_or_default()),
            ["perso", word, value] => match (
                value.parse::<i32>(),
                PERSO_OPTS.iter().find(|o| o.0 == word),
            ) {
                (Ok(v), Some(o)) if (-1..=o.2).contains(&v) => {
                    let p = st.perso.entry(uuid).or_default();
                    match o.0 {
                        "sons" => p.volume = v,
                        "particules" => p.particles = v,
                        _ => p.preview = v,
                    }
                    st.dirty = true;
                    perso_snapshot(st);
                    perso_page(&st.config, st.perso.get(&uuid).copied().unwrap_or_default())
                }
                _ => vec![span(
                    "🪓 Chop! Réglages perso : sons (0 à 200), particules (0 à 3), apercu (0 ou 1), -1 pour suivre le monde.",
                    "red",
                )],
            },
            ["stats"] => stats_lines(st, uuid),
            _ => help(admin),
        }
    });
    let lines = reply.unwrap_or_else(|| vec![span("🪓 Chop! est occupé, réessaie.", "red")]);
    for line in lines {
        let _ = host.commands().feedback(line);
    }
    true
}

fn toggled(off: bool) -> Text {
    let head = bold("🪓 Chop! ", "gold");
    if off {
        head.append(span("désactivé", "red"))
            .append(span(" pour toi : tu coupes bûche par bûche. ", "gray"))
            .append(button("[Réactiver]", "green", "/chop on", "/chop on"))
    } else {
        head.append(span("activé", "green"))
            .append(span(" pour toi : une bûche coupée abat l'arbre. ", "gray"))
            .append(button("[Désactiver]", "red", "/chop off", "/chop off"))
    }
}

fn help(admin: bool) -> Vec<Text> {
    let mut lines = vec![
        bold("🪓 Chop! ", "gold").append(span(
            "Coupe une bûche à la hache : tout l'arbre tombe.",
            "gray",
        )),
        button("/chop", "yellow", "Cliquer pour basculer", "/chop")
            .append(span(" active ou désactive Chop! pour toi.", "gray")),
        button("/chop perso", "yellow", "Tes réglages", "/chop perso")
            .append(span(" tes propres sons, particules et aperçu.", "gray")),
        button("/chop stats", "yellow", "Tes arbres", "/chop stats")
            .append(span(" tes arbres abattus et le classement.", "gray")),
    ];
    if admin {
        lines.push(
            button(
                "/chop config",
                "yellow",
                "Ouvrir les réglages",
                "/chop config",
            )
            .append(span(" ouvre les réglages (cliquables).", "gray")),
        );
        lines.push(
            span("/chop set <réglage> <valeur>", "yellow")
                .append(span(" change un réglage.", "gray")),
        );
        lines.push(
            span("/chop reset", "yellow").append(span(" remet les réglages par défaut.", "gray")),
        );
    }
    lines
}

/// A player's own settings: the word, the label, the highest value, the step and
/// how the value reads.
const PERSO_OPTS: [(&str, &str, i32, i32, Unit); 3] = [
    ("sons", "Volume des sons", 200, 25, Unit::Percent),
    (
        "particules",
        "Particules",
        3,
        1,
        Unit::Choice(rules::PARTICLES),
    ),
    ("apercu", "Aperçu de la chute", 1, 1, Unit::Switch),
];

/// `/chop perso`: the player's own settings, each one either theirs or the
/// world's.
fn perso_page(config: &Config, p: Perso) -> Vec<Text> {
    let mut lines = vec![bold("🪓 Chop! ", "gold").append(bold("[Tes réglages]", "white"))];
    for (word, label, max, step, unit) in PERSO_OPTS {
        let (own, world) = match word {
            "sons" => (p.volume, config.get(Key::Volume)),
            "particules" => (p.particles, config.get(Key::Particles)),
            _ => (p.preview, config.get(Key::Preview)),
        };
        let v = if own >= 0 { own } else { world };
        let mut line = span("  ", "gray")
            .append(span(label, "gray"))
            .append(span(" : ", "dark_gray"));
        if unit == Unit::Switch {
            let (yes, no) = if v != 0 {
                ("green", "dark_gray")
            } else {
                ("dark_gray", "red")
            };
            line = line
                .append(button(
                    "[oui]",
                    yes,
                    "oui",
                    &format!("/chop perso {word} 1"),
                ))
                .append(span(" ", "gray"))
                .append(button("[non]", no, "non", &format!("/chop perso {word} 0")));
        } else {
            let less = (v - step).max(0);
            let more = (v + step).min(max);
            line = line
                .append(button(
                    "[-]",
                    "red",
                    &unit.show(less),
                    &format!("/chop perso {word} {less}"),
                ))
                .append(span(" ", "gray"))
                .append(bold(&unit.show(v), "yellow"))
                .append(span(" ", "gray"))
                .append(button(
                    "[+]",
                    "green",
                    &unit.show(more),
                    &format!("/chop perso {word} {more}"),
                ));
        }
        line = if own >= 0 {
            line.append(span("  ", "gray")).append(button(
                "[comme le monde]",
                "aqua",
                &format!("Suivre le réglage du monde : {}", unit.show(world)),
                &format!("/chop perso {word} -1"),
            ))
        } else {
            line.append(span("  (comme le monde)", "dark_gray"))
        };
        lines.push(line);
    }
    lines
}

/// `/chop stats`: the player's trees and the leaderboard.
fn stats_lines(st: &State, uuid: u128) -> Vec<Text> {
    let mut lines = Vec::new();
    match st.stats.get(&uuid) {
        Some(s) => lines.push(
            bold("🪓 Chop! ", "gold")
                .append(span("Tes arbres : ", "gray"))
                .append(bold(&s.trees.to_string(), "yellow"))
                .append(span(
                    &format!(
                        " ({} bûches, {} blocs) · le plus grand : {} blocs · chaîne record : {}",
                        s.logs, s.blocks, s.biggest, s.chain
                    ),
                    "gray",
                )),
        ),
        None => lines.push(
            bold("🪓 Chop! ", "gold").append(span("Tu n'as encore abattu aucun arbre.", "gray")),
        ),
    }
    let mut board: Vec<&Stats> = st.stats.values().filter(|s| s.trees > 0).collect();
    board.sort_by(|a, b| b.trees.cmp(&a.trees).then(b.biggest.cmp(&a.biggest)));
    if !board.is_empty() {
        lines.push(span("  Classement :", "gold"));
        for (k, s) in board.iter().take(5).enumerate() {
            let medal = ["🥇", "🥈", "🥉", " 4.", " 5."][k];
            lines.push(span(
                &format!(
                    "  {medal} {} — {} arbres, record {} blocs",
                    s.name, s.trees, s.biggest
                ),
                if k == 0 { "yellow" } else { "gray" },
            ));
        }
    }
    lines
}

/// A page of the settings menu: tabs, one line per setting with its buttons.
fn page(config: &Config, section: Section) -> Vec<Text> {
    let mut tabs = bold("🪓 Chop! ", "gold");
    for s in Section::ALL {
        let label = format!("[{}]", s.title());
        tabs = if s == section {
            tabs.append(bold(&label, "white"))
        } else {
            tabs.append(button(
                &label,
                "dark_gray",
                &format!("Page {}", s.title()),
                &format!("/chop config {}", s.word()),
            ))
        }
        .append(span(" ", "gray"));
    }
    let mut lines = vec![tabs];
    if !config.on(Key::Active) && section != Section::Rules {
        lines.push(span(
            "  Chop! est désactivé sur ce monde (page Règles).",
            "red",
        ));
    }
    for o in rules::OPTS.iter().filter(|o| o.section == section) {
        let v = config.get(o.key);
        let mut line = span("  ", "gray")
            .append(hinted(o.label, "gray", o.help))
            .append(span(" : ", "dark_gray"));
        if o.unit == Unit::Switch {
            let (yes, no) = if v != 0 {
                ("green", "dark_gray")
            } else {
                ("dark_gray", "red")
            };
            line = line
                .append(button(
                    "[oui]",
                    yes,
                    &format!("{} : oui", o.label),
                    &format!("/chop set {} 1", o.word),
                ))
                .append(span(" ", "gray"))
                .append(button(
                    "[non]",
                    no,
                    &format!("{} : non", o.label),
                    &format!("/chop set {} 0", o.word),
                ));
        } else {
            let less = (v - o.step).max(o.min);
            let more = (v + o.step).min(o.max);
            let minus = if v > o.min {
                button(
                    "[-]",
                    "red",
                    &o.unit.show(less),
                    &format!("/chop set {} {less}", o.word),
                )
            } else {
                span("[-]", "dark_gray")
            };
            let plus = if v < o.max {
                button(
                    "[+]",
                    "green",
                    &o.unit.show(more),
                    &format!("/chop set {} {more}", o.word),
                )
            } else {
                span("[+]", "dark_gray")
            };
            line = line
                .append(minus)
                .append(span(" ", "gray"))
                .append(bold(&o.unit.show(v), "yellow"))
                .append(span(" ", "gray"))
                .append(plus);
        }
        lines.push(line);
    }
    lines.push(
        span("  ", "gray")
            .append(button(
                "[Tout remettre par défaut]",
                "dark_red",
                "Remet tous les réglages de Chop! par défaut",
                "/chop reset",
            ))
            .append(span("  ", "gray"))
            .append(button("[Aide]", "aqua", "Les commandes", "/chop aide")),
    );
    lines
}
