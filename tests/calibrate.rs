// cargo test --test calibrate -- --ignored --nocapture
use chop_horizon::*;
use std::collections::HashMap;

struct W(HashMap<Pos, Cell>);
impl World for W {
    fn cell(&mut self, p: Pos) -> Option<Cell> {
        Some(match self.0.get(&p) {
            Some(c) => *c,
            None if p[1] < 64 => Cell::Solid,
            None => Cell::Open,
        })
    }
}

fn tree(h: i32, r: i32) -> W {
    let mut m = HashMap::new();
    for y in 0..h {
        m.insert([0, 64 + y, 0], Cell::Log { wood: 1 });
    }
    let top = 64 + h;
    for dy in -(h / 2)..=1 {
        for dx in -r..=r {
            for dz in -r..=r {
                let p = [dx, top + dy, dz];
                if dx * dx + dz * dz > r * r + 1 || m.contains_key(&p) {
                    continue;
                }
                let d = (dx.abs().max(dz.abs()) + dy.max(0)).clamp(1, 6) as u8;
                m.insert(
                    p,
                    Cell::Leaves {
                        distance: d,
                        persistent: false,
                    },
                );
            }
        }
    }
    W(m)
}

#[test]
#[ignore]
fn calibrate() {
    for (h, r, speed) in [
        (5, 2, 200),
        (6, 2, 200),
        (8, 3, 200),
        (14, 3, 200),
        (25, 4, 200),
        (6, 2, 100),
        (25, 4, 100),
        (6, 2, 400),
    ] {
        let mut w = tree(h, r);
        w.0.insert([0, 64, 0], Cell::Open);
        let lim = Limits {
            max_logs: 512,
            max_blocks: 4000,
            radius: 24,
        };
        let t0 = std::time::Instant::now();
        let t = scan(&mut Look::new(&mut w), &|_| false, [0, 64, 0], 1, &lim).unwrap();
        let sink = gap(&mut Look::new(&mut w), &t);
        let d = choose_dir(&t, &mut Look::new(&mut w), 0.0, true, 30, sink);
        let b = Body::new(&t, d, sink);
        let c = b.contact(&mut Look::new(&mut w), true);
        let crush = b.crush(&mut Look::new(&mut w), c.theta);
        let el = t0.elapsed();
        let crushed = crush.iter().filter(|a| a.is_some()).count();
        let m = Motion::new(&b, &c, speed);
        let mut f = Fell::new(b, m, crush, 50, false, speed);
        let mut first = 0;
        let mut settle = 0;
        let mut imp = vec![];
        while !matches!(f.phase, Phase::Lying(_) | Phase::Breaking | Phase::Done) {
            let fr = f.tick();
            if let Some(i) = fr.impact {
                if i.first {
                    first = f.age;
                }
                imp.push(format!("{:.2}", i.omega));
            }
            settle = f.age;
        }
        let mut end = 0;
        while f.phase != Phase::Done {
            f.tick();
            end = f.age;
        }
        println!(
            "h {h:2} r {r} speed {speed}: blocks {:4} crushed {crushed:3} contact {:5.1}° first hit {:.2}s settled {:.2}s all gone {:.2}s impacts {:?} (setup {:?})",
            t.blocks(),
            c.theta.to_degrees(),
            first as f64 * TICK,
            settle as f64 * TICK,
            end as f64 * TICK,
            imp,
            el
        );
    }
}
