// The rules on worlds built by hand.

use chop_horizon::*;
use std::collections::{HashMap, HashSet, VecDeque};

const GROUND: i32 = 64;
const OAK: u16 = 1;
const BIRCH: u16 = 2;

/// Ground below y 64, air above, and what the test puts there.
#[derive(Default)]
struct W {
    cells: HashMap<Pos, Cell>,
    unloaded_x: Option<i32>,
}

impl World for W {
    fn cell(&mut self, p: Pos) -> Option<Cell> {
        if self.unloaded_x.is_some_and(|x| p[0] >= x) {
            return None;
        }
        Some(match self.cells.get(&p) {
            Some(c) => *c,
            None if p[1] < GROUND => Cell::Solid,
            None => Cell::Open,
        })
    }
}

impl W {
    fn set(&mut self, p: Pos, c: Cell) {
        self.cells.insert(p, c);
    }
    /// A round tree: a trunk of `height` logs on the ground at (x, z) and a crown of
    /// leaves around its top.
    fn tree(&mut self, x: i32, z: i32, height: i32, wood: u16) {
        for y in 0..height {
            self.set([x, GROUND + y, z], Cell::Log { wood });
        }
        let top = GROUND + height;
        for dy in -3..=1 {
            let r = if dy >= 0 { 1 } else { 2 };
            for dx in -r..=r {
                for dz in -r..=r {
                    let p = [x + dx, top + dy, z + dz];
                    if dx.abs() == r && dz.abs() == r && r == 2 && dy == -1 {
                        continue;
                    }
                    if !self.cells.contains_key(&p) {
                        self.set(
                            p,
                            Cell::Leaves {
                                distance: 7,
                                persistent: false,
                            },
                        );
                    }
                }
            }
        }
        self.distances();
    }
    /// Vanilla's leaf `distance`: steps to the nearest log through leaves, 7 at most.
    fn distances(&mut self) {
        let mut d: HashMap<Pos, u8> = HashMap::new();
        let mut q = VecDeque::new();
        for (p, c) in &self.cells {
            if matches!(c, Cell::Log { .. }) {
                d.insert(*p, 0);
                q.push_back(*p);
            }
        }
        while let Some(p) = q.pop_front() {
            let n = d[&p];
            if n >= 7 {
                continue;
            }
            for o in [
                [1, 0, 0],
                [-1, 0, 0],
                [0, 1, 0],
                [0, -1, 0],
                [0, 0, 1],
                [0, 0, -1],
            ] {
                let r = [p[0] + o[0], p[1] + o[1], p[2] + o[2]];
                if matches!(self.cells.get(&r), Some(Cell::Leaves { .. })) && !d.contains_key(&r) {
                    d.insert(r, n + 1);
                    q.push_back(r);
                }
            }
        }
        for (p, c) in self.cells.iter_mut() {
            if let Cell::Leaves { distance, .. } = c {
                *distance = d.get(p).copied().unwrap_or(7).min(7);
            }
        }
    }
    /// Cuts the log at `p` (the player broke it) and scans.
    fn cut(
        &mut self,
        p: Pos,
        wood: u16,
        placed: &HashSet<Pos>,
        limits: &Limits,
    ) -> Result<Tree, Refusal> {
        self.cells.insert(p, Cell::Open);
        let mut look = Look::new(self);
        scan(&mut look, &|q| placed.contains(&q), p, wood, limits)
    }
}

fn limits() -> Limits {
    Limits::of(&Config::default())
}

fn none() -> HashSet<Pos> {
    HashSet::new()
}

#[test]
fn settings_have_defaults_ranges_and_round_trip() {
    let mut c = Config::default();
    assert_eq!(c.get(Key::Speed), 200);
    assert!(
        c.on(Key::Monsters) && !c.on(Key::Animals) && !c.on(Key::Allies) && !c.on(Key::Players)
    );
    assert!(c.on(Key::Aim));
    assert_eq!(c.get(Key::Cone), 30);
    assert!(c.set("vitesse", 1000).is_err());
    assert!(c.set("nope", 1).is_err());
    assert_eq!(c.set("vitesse", 300).unwrap().key, Key::Speed);
    c.set("joueurs", 1).unwrap();
    c.set("pose", 40).unwrap();
    let lines = c.to_lines();
    assert_eq!(lines.len(), 3);
    let mut d = Config::default();
    for l in &lines {
        d.read_line(l);
    }
    d.read_line("garbage");
    d.read_line("angle = 999");
    assert_eq!(c, d);
    for (i, o) in OPTS.iter().enumerate() {
        assert_eq!(o.key as usize, i, "{} out of order", o.word);
        assert!((o.min..=o.max).contains(&o.default), "{}", o.word);
        assert!(o.word.is_ascii() && !o.word.contains(' '));
    }
    assert_eq!(Unit::Tenths.show(25), "2,5 s");
    assert_eq!(Unit::Health.show(10), "5 ❤");
    assert_eq!(Unit::Health.show(7), "3,5 ❤");
    assert_eq!(Key::Sneak.opt().unit.show(2), "toujours abattre");
}

#[test]
fn who_fells() {
    let c = Config::default();
    assert!(fells(&c, 0, false, true));
    assert!(!fells(&c, 0, true, true), "sneaking keeps one log");
    assert!(!fells(&c, 0, false, false), "an axe is needed");
    assert!(!fells(&c, 3, false, true) && !fells(&c, 2, false, true));
    assert!(fells(&c, 1, false, true), "creative fells too");
    let mut c2 = c.clone();
    c2.set("accroupi", 1).unwrap();
    assert!(fells(&c2, 0, true, true) && !fells(&c2, 0, false, true));
    c2.set("accroupi", 2).unwrap();
    c2.set("hache", 0).unwrap();
    assert!(fells(&c2, 0, true, false) && fells(&c2, 0, false, false));
    c2.set("actif", 0).unwrap();
    assert!(!fells(&c2, 0, false, true));
}

#[test]
fn a_tree_is_its_trunk_and_crown() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    let crown: usize = w
        .cells
        .values()
        .filter(|c| matches!(c, Cell::Leaves { .. }))
        .count();
    let t = w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    assert_eq!(t.logs.len(), 5);
    assert!(t.logs.iter().all(|p| p[1] > GROUND));
    assert_eq!(t.leaves.len(), crown);
    assert!(t.hangers.is_empty());
}

#[test]
fn a_neighbour_keeps_its_leaves_and_logs() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    w.tree(4, 0, 6, BIRCH);
    let t = w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    assert!(t.logs.iter().all(|p| p[0] == 0));
    // no leaf of the birch: each claimed leaf is nearer to the oak's trunk
    for p in &t.leaves {
        assert!(p[0] <= 2, "{p:?} belongs to the birch");
    }
    let birch = w.cut([4, GROUND, 0], BIRCH, &none(), &limits()).unwrap();
    // halfway leaves (x = 2) are as near to both: either tree may take them
    assert!(birch.leaves.iter().all(|p| p[0] >= 2));
    assert!(birch.logs.iter().all(|p| p[0] == 4));
}

#[test]
fn branches_houses_placed_logs_and_giants_stay() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    // a branch: nothing under the cut
    w.set([1, GROUND + 3, 0], Cell::Log { wood: OAK });
    w.distances();
    assert_eq!(
        w.cut([1, GROUND + 3, 0], OAK, &none(), &limits())
            .unwrap_err(),
        Refusal::Branch
    );
    // a placed log in the trunk
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    let placed = HashSet::from([[0, GROUND + 3, 0]]);
    assert_eq!(
        w.cut([0, GROUND, 0], OAK, &placed, &limits()).unwrap_err(),
        Refusal::Placed
    );
    // a log pillar without leaves
    let mut w = W::default();
    for y in 0..5 {
        w.set([0, GROUND + y, 0], Cell::Log { wood: OAK });
    }
    assert_eq!(
        w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap_err(),
        Refusal::NotATree
    );
    // too big
    let mut w = W::default();
    w.tree(0, 0, 12, OAK);
    let small = Limits {
        max_logs: 8,
        ..limits()
    };
    assert_eq!(
        w.cut([0, GROUND, 0], OAK, &none(), &small).unwrap_err(),
        Refusal::TooBig
    );
    // the level not loaded
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    w.unloaded_x = Some(1);
    assert_eq!(
        w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap_err(),
        Refusal::Unloaded
    );
}

#[test]
fn what_hangs_on_it_breaks_off() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    w.set([0, GROUND + 2, 1], Cell::Hanger(Attach::Side)); // vine on the trunk
    w.set([0, GROUND + 1, 1], Cell::Hanger(Attach::Side)); // the vine below it
    w.set([0, GROUND + 8, 0], Cell::Hanger(Attach::Above)); // snow on the crown
    w.set([2, GROUND + 2, 0], Cell::Hanger(Attach::Below)); // moss under a leaf
    w.set([3, GROUND, 0], Cell::Hanger(Attach::Above)); // snow on the ground: not the tree's
    let t = w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    let h: HashSet<Pos> = t.hangers.iter().copied().collect();
    assert!(h.contains(&[0, GROUND + 2, 1]) && h.contains(&[0, GROUND + 1, 1]));
    assert!(h.contains(&[0, GROUND + 8, 0]));
    assert!(h.contains(&[2, GROUND + 2, 0]));
    assert!(!h.contains(&[3, GROUND, 0]));
}

/// The oak at the origin cut at the ground, its body falling along `dir`, dropped
/// into the cut's gap as the rules plan it.
fn planned(w: &mut W, dir: [f64; 2]) -> (Tree, Body) {
    let t = w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    let sink = gap(&mut Look::new(w), &t);
    let b = Body::new(&t, dir, sink);
    (t, b)
}

fn part_at(b: &Body, cell: Pos) -> usize {
    b.parts.iter().position(|p| p.cell == cell).unwrap()
}

#[test]
fn the_body_turns_about_the_front_of_its_base() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    let t = w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    let b = Body::new(&t, [1.0, 0.0], 0.0);
    assert_eq!(b.pivot, [1.0, (GROUND + 1) as f64, 0.5]);
    // upright, a block shows where it is
    let anchor = [0.5, GROUND as f64 + 0.5, 0.5];
    let tr = b.translation([0, GROUND + 3, 0], 0.0, anchor);
    assert!((tr[0] + 0.5).abs() < 1e-6 && (tr[1] - 2.5).abs() < 1e-6 && (tr[2] + 0.5).abs() < 1e-6);
    // a quarter turn lays the trunk along +x at the hinge's height
    let q = std::f64::consts::FRAC_PI_2;
    let top = b.world(b.local([0.5, GROUND as f64 + 5.5, 0.5]), q);
    assert!((top[0] - (1.0 + 4.5)).abs() < 1e-9 && (top[1] - (GROUND as f64 + 1.5)).abs() < 1e-9);
    // the quaternion turns up toward the fall
    let [x, y, z, s] = b.quat(q).map(f64::from);
    // the rotation matrix's second column: where (0, 1, 0) goes
    let up = [
        2.0 * (x * y - s * z),
        1.0 - 2.0 * (x * x + z * z),
        2.0 * (y * z + s * x),
    ];
    assert!((up[0] - 1.0).abs() < 1e-6 && up[1].abs() < 1e-6 && up[2].abs() < 1e-6);
    // and the display's corner plus the turned unit cube lands where world() says
    let tr = b.translation([0, GROUND + 3, 0], q, anchor);
    let corner = b.world(b.local([0.0, (GROUND + 3) as f64, 0.0]), q);
    for k in 0..3 {
        assert!((anchor[k] + f64::from(tr[k]) - corner[k]).abs() < 1e-4);
    }
    // pose() at its size is the same, and a puffed-up block keeps its centre
    let i = part_at(&b, [0, GROUND + 3, 0]);
    let (tr2, q2) = b.pose(i, q, anchor, 1.0, 0.0);
    for k in 0..3 {
        assert!((tr2[k] - tr[k]).abs() < 1e-4);
    }
    assert_eq!(q2, b.quat(q));
    let (tr3, _) = b.pose(i, q, anchor, 1.4, 0.25);
    let half = b.turn([0.7; 3], q);
    let c = b.center(i, q);
    for k in 0..3 {
        let lift = if k == 1 { 0.25 } else { 0.0 };
        assert!((anchor[k] + f64::from(tr3[k]) + half[k] - c[k] - lift).abs() < 1e-4);
    }
    assert!(b.mv > 0.0 && b.inertia > 0.0 && b.mu < 0.0);
}

#[test]
fn the_trunk_drops_into_the_cut_then_turns() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    let (_, b) = planned(&mut w, [1.0, 0.0]);
    assert_eq!(b.sink, 1.0);
    // a 2×2 trunk cut on one side still stands on the three others
    let mut w2 = W::default();
    for x in 0..2 {
        for z in 0..2 {
            w2.tree(x, z, 8, OAK);
        }
    }
    let t2 = w2.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    assert_eq!(gap(&mut Look::new(&mut w2), &t2), 0.0);
    // it drops a block, lands on the stump, then turns
    let c = b.contact(&mut Look::new(&mut w), true);
    let crush = b.crush(&mut Look::new(&mut w), c.theta);
    let m = Motion::new(&b, &c, 200);
    let top = part_at(&b, [0, GROUND + 5, 0]);
    let mut f = Fell::new(b, m, crush, 10, false, 200);
    assert_eq!(f.phase, Phase::Dropping);
    let y0 = f.body.center(top, 0.0)[1];
    let mut landed = None;
    for _ in 0..20 {
        let fr = f.tick();
        assert!(fr.moved && fr.theta == 0.0);
        if fr.landed.is_some() {
            landed = fr.landed;
            break;
        }
    }
    let v = landed.expect("it lands on the stump");
    assert!(v > 2.0, "{v}");
    assert!(f.age <= 10, "a block's drop takes {} ticks", f.age);
    assert!((f.body.center(top, 0.0)[1] - (y0 - 1.0)).abs() < 1e-9);
    assert_eq!(f.phase, Phase::Falling);
    assert!(f.tick().theta > 0.0, "and it starts turning at once");
}

#[test]
fn it_lies_down_on_flat_ground_and_stops_on_a_wall() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    let (_, b) = planned(&mut w, [1.0, 0.0]);
    let c = b.contact(&mut Look::new(&mut w), true);
    assert!(c.ground);
    let deg = c.theta.to_degrees();
    assert!((80.0..=100.0).contains(&deg), "lies at {deg}°");
    assert!(!c.points.is_empty());
    // a wall 3 blocks ahead
    for y in GROUND..GROUND + 6 {
        for z in -3..=3 {
            w.set([4, y, z], Cell::Solid);
        }
    }
    let c2 = b.contact(&mut Look::new(&mut w), true);
    assert!(
        c2.ground && c2.theta < c.theta - 0.3,
        "{} vs {}",
        c2.theta.to_degrees(),
        deg
    );
    // the trunk's touch on the wall says which blocks it hit
    assert!(c2.points.iter().any(|(_, cell)| cell[0] == 4));
    // a cliff right in front of the stump: nothing to stop it
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    for x in 1..40 {
        for z in -10..=10 {
            for y in GROUND - 40..GROUND {
                w.set([x, y, z], Cell::Open);
            }
        }
    }
    let (_, b) = planned(&mut w, [1.0, 0.0]);
    let c3 = b.contact(&mut Look::new(&mut w), true);
    assert!(!c3.ground && (c3.theta - MAX_ANGLE).abs() < 1e-9);
}

#[test]
fn leaves_are_crushed_instead_of_holding_the_tree_up() {
    // a short trunk under a wide crown: the leaves reach the ground first
    let mut w = W::default();
    w.tree(0, 0, 4, OAK);
    let (_, b) = planned(&mut w, [1.0, 0.0]);
    let c = b.contact(&mut Look::new(&mut w), true);
    assert!(c.theta.to_degrees() > 80.0, "{}", c.theta.to_degrees());
    let crush = b.crush(&mut Look::new(&mut w), c.theta);
    let n = crush.iter().filter(|a| a.is_some()).count();
    assert!(n >= 4, "{n} leaves crushed");
    assert!(
        crush
            .iter()
            .zip(&b.parts)
            .all(|(a, p)| a.is_none() || p.leaf)
    );
    assert!(crush.iter().flatten().all(|a| *a <= c.theta + 1e-9));
    // in the fall each of them goes once, by the time the trunk lands
    let m = Motion::new(&b, &c, 200);
    let mut f = Fell::new(b, m, crush, 10, false, 200);
    let mut gone = 0;
    let mut hit = false;
    while f.phase != Phase::Done && f.age < 400 {
        let fr = f.tick();
        if fr.impact.is_some() {
            hit = true;
        }
        if hit && fr.impact.is_none() {
            assert!(fr.crushed.is_empty(), "crushed after the impact");
        }
        gone += fr.crushed.len();
        for i in &fr.broke {
            assert!(!fr.crushed.contains(i));
        }
    }
    assert_eq!(gone, n);
}

#[test]
fn the_cut_block_shakes_then_pops() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    let (_, b) = planned(&mut w, [0.0, 1.0]);
    let c = b.contact(&mut Look::new(&mut w), true);
    let crush = b.crush(&mut Look::new(&mut w), c.theta);
    let m = Motion::new(&b, &c, 200);
    let mut f = Fell::new(b, m, crush, 10, true, 200);
    for k in 0..SHAKE_TICKS {
        let fr = f.tick();
        let (tick, scale) = fr.shake.expect("it shakes");
        assert_eq!(tick, k);
        assert!(!fr.moved);
        assert_eq!(fr.popped, k == SHAKE_TICKS - 1);
        // the ghost's centre stays within a jiggle of the block's
        let (tr, q) = shake_pose(tick, scale);
        let (qy, qw) = (f64::from(q[1]), f64::from(q[3]));
        let yaw = 2.0 * qy.atan2(qw);
        let h = 0.5 * scale;
        let (ys, yc) = yaw.sin_cos();
        let centre = [
            f64::from(tr[0]) + h * yc + h * ys,
            f64::from(tr[1]) + h,
            f64::from(tr[2]) - h * ys + h * yc,
        ];
        assert!(centre.iter().all(|v| v.abs() < 0.08), "{centre:?}");
    }
    assert_eq!(f.phase, Phase::Dropping);
}

/// A whole felling: the ticks until the first impact (drop included), the impacts,
/// the felling, the order the blocks began to pop in and the tune's notes.
fn run(height: i32, speed: i32) -> (u32, Vec<Impact>, Fell, Vec<usize>, Vec<usize>) {
    let mut w = W::default();
    w.tree(0, 0, height, OAK);
    let (_, b) = planned(&mut w, [0.0, 1.0]);
    let c = b.contact(&mut Look::new(&mut w), true);
    let crush = b.crush(&mut Look::new(&mut w), c.theta);
    let m = Motion::new(&b, &c, speed);
    let mut f = Fell::new(b, m, crush, 50, false, speed);
    let mut impacts = Vec::new();
    let mut first_hit = 0;
    let n = f.alive.len();
    let mut broken = vec![0; n];
    let mut crushed = vec![false; n];
    let mut popped = vec![0; n];
    let mut order = Vec::new();
    let mut notes = Vec::new();
    for _ in 0..3000 {
        let fr = f.tick();
        if let Some(i) = fr.impact {
            if impacts.is_empty() {
                first_hit = f.age;
            }
            impacts.push(i);
        }
        for i in fr.crushed {
            crushed[i] = true;
        }
        for (i, scale, lift) in fr.popping {
            if popped[i] == 0 {
                order.push(i);
            }
            popped[i] += 1;
            assert!(scale > 0.0 && lift >= 0.0);
        }
        for i in fr.broke {
            assert_eq!(popped[i], POP.len(), "it puffed up first");
            broken[i] += 1;
        }
        if let Some(note) = fr.note {
            notes.push(note);
        }
        if fr.done {
            break;
        }
    }
    for i in 0..n {
        let expect = usize::from(!crushed[i]);
        assert_eq!(
            broken[i], expect,
            "each block that wasn't crushed breaks once"
        );
    }
    assert_eq!(f.phase, Phase::Done);
    (first_hit, impacts, f, order, notes)
}

#[test]
fn a_fall_speeds_up_hits_bounces_and_breaks_from_the_base_up() {
    let (ticks, impacts, f, order, notes) = run(6, 200);
    let secs = ticks as f64 * TICK;
    assert!((0.5..=2.0).contains(&secs), "a small oak falls in {secs} s");
    assert!(impacts[0].first && impacts[0].ground && impacts[0].omega > 1.0);
    assert!(impacts.iter().filter(|i| i.first).count() == 1);
    assert!(impacts.last().unwrap().last);
    assert!(impacts.len() >= 2, "it bounces");
    for pair in impacts.windows(2) {
        assert!(pair[1].omega < pair[0].omega);
    }
    assert!(f.remaining().count() == 0);
    // the base goes first, the top last, to a rising tune
    let r = |i: usize| f.body.parts[i].r;
    for pair in order.windows(2) {
        assert!(r(pair[0]) <= r(pair[1]) + 1e-9);
    }
    assert!(!notes.is_empty() && notes.windows(2).all(|p| p[0] <= p[1]));
    assert!(notes.iter().all(|n| *n < TUNE.len()));
    // taller is slower, faster gravity is quicker
    let (tall, ..) = run(14, 200);
    assert!(tall > ticks, "{tall} vs {ticks}");
    let (quick, ..) = run(6, 400);
    let (slow, ..) = run(6, 100);
    assert!(quick < ticks && ticks < slow, "{quick} {ticks} {slow}");
}

#[test]
fn it_goes_on_when_what_stopped_it_gives_way() {
    let mut w = W::default();
    w.tree(0, 0, 8, OAK);
    for y in GROUND..GROUND + 8 {
        for z in -3..=3 {
            w.set([4, y, z], Cell::Solid);
        }
    }
    let (_, b) = planned(&mut w, [1.0, 0.0]);
    let c = b.contact(&mut Look::new(&mut w), true);
    let crush = b.crush(&mut Look::new(&mut w), c.theta);
    let m = Motion::new(&b, &c, 200);
    let mut f = Fell::new(b, m, crush, 10, false, 200);
    let first = loop {
        if let Some(i) = f.tick().impact {
            break i;
        }
        assert!(f.age < 200);
    };
    let stopped = f.theta();
    assert!(stopped < 1.2);
    // the wall goes (a tree it knocked down): it falls on to the ground
    for y in GROUND..GROUND + 8 {
        for z in -3..=3 {
            w.cells.remove(&[4, y, z]);
        }
    }
    let c2 = f.body.contact(&mut Look::new(&mut w), true);
    let crush = f.body.crush(&mut Look::new(&mut w), c2.theta);
    assert!(c2.theta > stopped + 0.3);
    f.resume(&c2, crush, first.omega * 0.5);
    assert_eq!(f.phase, Phase::Falling);
    let again = loop {
        if let Some(i) = f.tick().impact {
            break i;
        }
        assert!(f.age < 400);
    };
    assert!(again.first && f.theta() > stopped + 0.3);
}

#[test]
fn it_strikes_what_is_in_the_way() {
    let mut w = W::default();
    w.tree(0, 0, 8, OAK);
    let (_, b) = planned(&mut w, [0.0, 1.0]);
    let c = b.contact(&mut Look::new(&mut w), true);
    let crush = b.crush(&mut Look::new(&mut w), c.theta);
    let m = Motion::new(&b, &c, 200);
    let mut f = Fell::new(b, m, crush, 10, false, 200);
    // a zombie-sized box 5 blocks ahead, standing on the ground
    let lo = [-0.3, GROUND as f64, 4.7];
    let hi = [0.3, GROUND as f64 + 1.95, 5.3];
    let mut hit = None;
    while matches!(f.phase, Phase::Dropping | Phase::Falling) {
        f.tick();
        let alive = f.alive.clone();
        if let Some(h) = f
            .body
            .strike(f.motion.theta, f.motion.omega, lo, hi, &alive)
        {
            hit = Some(h);
            break;
        }
    }
    let (speed, _leaf) = hit.expect("the trunk comes down on it");
    assert!(speed > 2.0, "{speed}");
    // nothing behind the tree is struck
    let behind = f.body.strike(
        f.motion.theta,
        3.0,
        [-0.3, 64.0, -5.3],
        [0.3, 66.0, -4.7],
        &f.alive.clone(),
    );
    assert!(behind.is_none());
}

#[test]
fn unbreaking_spares_wear() {
    let mut rng = Rng::new(42);
    assert_eq!(wear(100, 0, &mut rng), 100);
    let n = wear(4000, 3, &mut rng);
    assert!((800..1200).contains(&n), "{n}");
}

#[test]
fn a_log_being_dug_trembles_but_stays_covered() {
    for face in [[0, 1, 0], [1, 0, 0], [0, 0, -1]] {
        for k in 0..12 {
            for strength in [0.0, 0.5, 1.0] {
                let (tr, scale) = tremble_pose(k, strength, face);
                let (tr, scale) = (tr.map(f64::from), f64::from(scale));
                // the display's box, from its entity at the centre of the next cell
                // out, relative to the dug block's corner
                for i in 0..3 {
                    let lo = 0.5 + f64::from(face[i]) + tr[i];
                    let hi = lo + scale;
                    assert!(lo <= 0.0 && hi >= 1.0, "face {face:?} k {k}: {lo}..{hi}");
                    assert!(lo > -0.1 && hi < 1.1, "it stays close: {lo}..{hi}");
                }
            }
        }
    }
    // it moves from tick to tick, and harder as the dig goes on
    let (a, _) = tremble_pose(0, 0.0, [0, 1, 0]);
    let (b, _) = tremble_pose(1, 0.0, [0, 1, 0]);
    let (c, s1) = tremble_pose(1, 1.0, [0, 1, 0]);
    let (_, s0) = tremble_pose(1, 0.0, [0, 1, 0]);
    assert!((a[0] - b[0]).abs() > 0.01);
    assert!(s1 > s0 && (c[0] - a[0]).abs() > (b[0] - a[0]).abs());
}

#[test]
fn a_tree_is_replanted_where_it_stood() {
    let mut w = W::default();
    w.tree(0, 0, 6, OAK);
    let t = w.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    assert_eq!(t.spots(), vec![[0, GROUND, 0]]);
    // a 2×2 trunk: its four places
    let mut w2 = W::default();
    for x in 0..2 {
        for z in 0..2 {
            w2.tree(x, z, 8, OAK);
        }
    }
    let t2 = w2.cut([0, GROUND, 0], OAK, &none(), &limits()).unwrap();
    let mut spots = t2.spots();
    spots.sort();
    assert_eq!(
        spots,
        vec![
            [0, GROUND, 0],
            [0, GROUND, 1],
            [1, GROUND, 0],
            [1, GROUND, 1]
        ]
    );
    assert_eq!(
        sapling("oak", Some("minecraft:oak_leaves")),
        Some("minecraft:oak_sapling")
    );
    assert_eq!(
        sapling("oak", Some("minecraft:azalea_leaves")),
        Some("minecraft:azalea")
    );
    assert_eq!(
        sapling("oak", Some("minecraft:flowering_azalea_leaves")),
        Some("minecraft:flowering_azalea")
    );
    assert_eq!(
        sapling("mangrove", None),
        Some("minecraft:mangrove_propagule")
    );
    assert_eq!(sapling("crimson", None), Some("minecraft:crimson_fungus"));
    assert_eq!(
        sapling("mushroom", Some("minecraft:red_mushroom_block")),
        Some("minecraft:red_mushroom")
    );
    assert_eq!(sapling("bamboo", None), None);
    assert_eq!(Config::default().get(Key::Replant), 1);
}

#[test]
fn a_huge_fungus_falls_with_its_caps() {
    const CRIMSON: u16 = 7;
    let mut w = W::default();
    for y in 0..6 {
        w.set([0, GROUND + y, 0], Cell::Log { wood: CRIMSON });
    }
    // a cap of wart blocks and a shroomlight round the top, weeping vines under it
    for dx in -2..=2i32 {
        for dz in -2..=2i32 {
            for dy in 4..=6 {
                let p = [dx, GROUND + dy, dz];
                if !w.cells.contains_key(&p) && (dx.abs() == 2 || dz.abs() == 2 || dy == 6) {
                    w.set(p, Cell::Cap);
                }
            }
        }
    }
    w.set([2, GROUND + 3, 0], Cell::Hanger(Attach::Below));
    // a wart block on the ground nearby is not the fungus's
    w.set([6, GROUND, 0], Cell::Cap);
    let t = w.cut([0, GROUND, 0], CRIMSON, &none(), &limits()).unwrap();
    assert_eq!(t.logs.len(), 5);
    assert!(t.leaves.len() > 20);
    assert!(!t.leaves.contains(&[6, GROUND, 0]));
    assert!(t.hangers.contains(&[2, GROUND + 3, 0]));
    let b = Body::new(&t, [1.0, 0.0], 1.0);
    assert!(b.parts.iter().filter(|p| p.leaf).count() == t.leaves.len());
}
