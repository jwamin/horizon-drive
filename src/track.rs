//! Stage themes and procedural stage layout (curves, hills, roadside props, forks).
use crate::consts::*;
use macroquad::prelude::*;

pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) | 1)
    }
    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
    pub fn f(&mut self) -> f32 {
        (self.next_u32() % 1_000_000) as f32 / 1_000_000.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    pub fn int(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n.max(1)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Prop {
    Palm,
    Pine,
    SnowPine,
    Cypress,
    Cactus,
    Building,
    Rock,
    Lamp,
    Billboard,
    Column,
    Turbine,
    Bush,
    Flowers,
    Lighthouse,
    Pole,
    Divider,
    ForkSign,
    Checkpoint,
    StartGate,
    GoalGate,
}

impl Prop {
    /// Half-width used for crashing into it (0 = drive through).
    pub fn hit_half(self) -> f32 {
        match self {
            Prop::Palm => 0.5,
            Prop::Pine | Prop::SnowPine => 0.7,
            Prop::Cypress => 0.7,
            Prop::Cactus => 0.6,
            Prop::Building => 6.0,
            Prop::Rock => 1.8,
            Prop::Lamp | Prop::Pole => 0.3,
            Prop::Billboard => 4.0,
            Prop::Column => 0.7,
            Prop::Turbine => 1.2,
            Prop::Bush => 1.2,
            Prop::Lighthouse => 2.5,
            Prop::Divider => 0.8,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PropInst {
    pub kind: Prop,
    pub x: f32,
    pub var: f32, // 0..1 variation (height, colour)
}

pub struct Seg {
    pub curve: f32,
    pub y: f32,
    pub props: Vec<PropInst>,
}

pub struct Theme {
    pub name: &'static str,
    pub sky_top: Color,
    pub sky_bot: Color,
    pub far: Color,
    pub near_hill: Color,
    pub ground: [Color; 2],
    pub rumble: [Color; 2],
    pub road: [Color; 2],
    pub lane: Color,
    pub fog: Color,
    pub sun: Color,
    pub sea: Option<Color>,
    pub props: &'static [Prop],
    pub curvy: f32,
    pub hilly: f32,
    pub traffic: f32,
    pub night: bool,
}

const fn c(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

const ROAD_A: [Color; 2] = [c(118, 118, 124), c(110, 110, 116)];
const WHITE_RED: [Color; 2] = [c(240, 240, 240), c(210, 40, 40)];

pub fn themes() -> Vec<Theme> {
    use Prop::*;
    vec![
        // Row 1
        Theme {
            name: "Palm Coast",
            sky_top: c(40, 120, 230), sky_bot: c(150, 210, 250), far: c(120, 160, 190), near_hill: c(90, 150, 110),
            ground: [c(230, 210, 150), c(220, 198, 138)], rumble: WHITE_RED, road: ROAD_A, lane: c(250, 250, 250),
            fog: c(170, 215, 245), sun: c(255, 250, 220), sea: Some(c(30, 130, 200)),
            props: &[Palm, Palm, Palm, Bush, Billboard], curvy: 0.6, hilly: 0.3, traffic: 1.0, night: false,
        },
        // Row 2
        Theme {
            name: "Vineyard Hills",
            sky_top: c(60, 140, 235), sky_bot: c(175, 220, 250), far: c(110, 150, 120), near_hill: c(80, 130, 60),
            ground: [c(110, 170, 70), c(98, 158, 62)], rumble: [c(240, 240, 240), c(40, 40, 40)], road: ROAD_A, lane: c(250, 250, 250),
            fog: c(185, 225, 245), sun: c(255, 250, 220), sea: None,
            props: &[Cypress, Cypress, Bush, Bush, Pole], curvy: 0.8, hilly: 0.9, traffic: 0.9, night: false,
        },
        Theme {
            name: "Lighthouse Cliffs",
            sky_top: c(30, 110, 220), sky_bot: c(160, 215, 250), far: c(100, 120, 140), near_hill: c(120, 120, 110),
            ground: [c(125, 150, 90), c(115, 140, 82)], rumble: WHITE_RED, road: ROAD_A, lane: c(250, 250, 250),
            fog: c(175, 215, 240), sun: c(255, 250, 225), sea: Some(c(20, 100, 180)),
            props: &[Rock, Rock, Bush, Lighthouse, Pole], curvy: 0.9, hilly: 0.6, traffic: 0.8, night: false,
        },
        // Row 3
        Theme {
            name: "Redwood Pass",
            sky_top: c(70, 140, 220), sky_bot: c(190, 225, 240), far: c(70, 100, 90), near_hill: c(40, 80, 50),
            ground: [c(70, 120, 55), c(62, 110, 50)], rumble: [c(240, 240, 240), c(40, 120, 40)], road: [c(100, 100, 104), c(94, 94, 98)], lane: c(245, 230, 120),
            fog: c(180, 210, 215), sun: c(255, 245, 210), sea: None,
            props: &[Pine, Pine, Pine, Rock], curvy: 1.0, hilly: 1.0, traffic: 0.8, night: false,
        },
        Theme {
            name: "Harbor Town",
            sky_top: c(50, 130, 230), sky_bot: c(200, 225, 245), far: c(140, 150, 170), near_hill: c(150, 140, 130),
            ground: [c(170, 165, 155), c(160, 155, 146)], rumble: [c(250, 250, 250), c(30, 90, 180)], road: [c(90, 90, 95), c(84, 84, 90)], lane: c(250, 250, 250),
            fog: c(200, 220, 240), sun: c(255, 250, 225), sea: Some(c(40, 110, 170)),
            props: &[Building, Building, Lamp, Lamp, Palm], curvy: 0.7, hilly: 0.4, traffic: 1.3, night: false,
        },
        Theme {
            name: "Salt Flats",
            sky_top: c(20, 100, 220), sky_bot: c(200, 230, 250), far: c(170, 160, 175), near_hill: c(200, 190, 190),
            ground: [c(240, 240, 235), c(232, 232, 226)], rumble: [c(250, 120, 30), c(250, 250, 250)], road: [c(70, 70, 75), c(64, 64, 70)], lane: c(250, 210, 60),
            fog: c(220, 235, 250), sun: c(255, 255, 240), sea: None,
            props: &[Pole, Pole, Rock, Billboard], curvy: 0.35, hilly: 0.2, traffic: 0.7, night: false,
        },
        // Row 4
        Theme {
            name: "Canyon Mesa",
            sky_top: c(60, 120, 210), sky_bot: c(240, 200, 150), far: c(170, 90, 60), near_hill: c(190, 110, 70),
            ground: [c(205, 130, 80), c(195, 122, 74)], rumble: [c(250, 250, 250), c(160, 60, 30)], road: [c(120, 100, 90), c(112, 94, 84)], lane: c(250, 240, 200),
            fog: c(235, 200, 160), sun: c(255, 240, 200), sea: None,
            props: &[Cactus, Cactus, Rock, Rock], curvy: 0.9, hilly: 0.8, traffic: 0.8, night: false,
        },
        Theme {
            name: "Neon Boulevard",
            sky_top: c(40, 20, 80), sky_bot: c(240, 110, 120), far: c(60, 40, 90), near_hill: c(50, 30, 70),
            ground: [c(70, 60, 90), c(64, 54, 84)], rumble: [c(255, 80, 200), c(60, 220, 255)], road: [c(55, 50, 70), c(50, 46, 64)], lane: c(255, 230, 120),
            fog: c(200, 100, 140), sun: c(255, 150, 120), sea: None,
            props: &[Building, Building, Lamp, Lamp, Billboard], curvy: 0.7, hilly: 0.3, traffic: 1.3, night: true,
        },
        Theme {
            name: "Windmill Plains",
            sky_top: c(60, 150, 240), sky_bot: c(190, 230, 250), far: c(120, 160, 140), near_hill: c(160, 170, 80),
            ground: [c(215, 195, 80), c(200, 182, 72)], rumble: [c(250, 250, 250), c(40, 40, 40)], road: ROAD_A, lane: c(250, 250, 250),
            fog: c(195, 230, 245), sun: c(255, 250, 220), sea: None,
            props: &[Turbine, Bush, Pole, Bush], curvy: 0.5, hilly: 0.7, traffic: 0.9, night: false,
        },
        Theme {
            name: "Lakeside Forest",
            sky_top: c(50, 120, 220), sky_bot: c(180, 220, 245), far: c(80, 110, 130), near_hill: c(50, 90, 70),
            ground: [c(80, 140, 70), c(72, 130, 64)], rumble: WHITE_RED, road: ROAD_A, lane: c(250, 250, 250),
            fog: c(180, 215, 235), sun: c(255, 250, 220), sea: Some(c(50, 120, 160)),
            props: &[Pine, Pine, Bush, Rock], curvy: 1.0, hilly: 0.7, traffic: 0.8, night: false,
        },
        // Row 5 (goals)
        Theme {
            name: "Snowline Summit",
            sky_top: c(80, 140, 220), sky_bot: c(215, 230, 245), far: c(200, 210, 230), near_hill: c(230, 235, 245),
            ground: [c(245, 248, 252), c(232, 238, 246)], rumble: WHITE_RED, road: [c(105, 108, 118), c(98, 101, 112)], lane: c(250, 210, 60),
            fog: c(225, 235, 248), sun: c(255, 255, 245), sea: None,
            props: &[SnowPine, SnowPine, Rock, Pole], curvy: 1.0, hilly: 1.0, traffic: 0.7, night: false,
        },
        Theme {
            name: "Desert Sunset",
            sky_top: c(90, 40, 110), sky_bot: c(255, 150, 70), far: c(130, 60, 70), near_hill: c(160, 80, 60),
            ground: [c(220, 150, 90), c(210, 140, 82)], rumble: [c(250, 250, 250), c(200, 60, 40)], road: [c(100, 80, 80), c(94, 75, 75)], lane: c(255, 230, 160),
            fog: c(250, 160, 100), sun: c(255, 200, 90), sea: None,
            props: &[Cactus, Rock, Cactus, Pole], curvy: 0.6, hilly: 0.5, traffic: 0.8, night: false,
        },
        Theme {
            name: "Ancient Ruins",
            sky_top: c(50, 130, 230), sky_bot: c(205, 230, 245), far: c(140, 150, 130), near_hill: c(150, 160, 110),
            ground: [c(160, 170, 100), c(150, 160, 92)], rumble: [c(240, 235, 220), c(180, 150, 100)], road: [c(140, 130, 115), c(132, 122, 108)], lane: c(250, 250, 240),
            fog: c(205, 228, 240), sun: c(255, 250, 225), sea: Some(c(40, 120, 190)),
            props: &[Column, Column, Cypress, Rock], curvy: 0.8, hilly: 0.6, traffic: 0.8, night: false,
        },
        Theme {
            name: "Tulip Fields",
            sky_top: c(70, 150, 240), sky_bot: c(200, 230, 250), far: c(130, 170, 160), near_hill: c(110, 170, 80),
            ground: [c(120, 185, 80), c(110, 175, 72)], rumble: [c(250, 250, 250), c(240, 120, 30)], road: ROAD_A, lane: c(250, 250, 250),
            fog: c(200, 232, 248), sun: c(255, 250, 220), sea: None,
            props: &[Flowers, Flowers, Flowers, Turbine, Bush], curvy: 0.6, hilly: 0.4, traffic: 0.9, night: false,
        },
        Theme {
            name: "Midnight Skyline",
            sky_top: c(5, 8, 30), sky_bot: c(40, 40, 90), far: c(20, 22, 45), near_hill: c(15, 15, 35),
            ground: [c(30, 32, 45), c(26, 28, 40)], rumble: [c(240, 240, 240), c(230, 40, 60)], road: [c(45, 45, 55), c(41, 41, 50)], lane: c(255, 220, 120),
            fog: c(30, 30, 70), sun: c(240, 240, 255), sea: None,
            props: &[Building, Building, Lamp, Lamp], curvy: 0.8, hilly: 0.4, traffic: 1.2, night: true,
        },
    ]
}

pub fn stage_row(id: usize) -> (usize, usize) {
    let mut r = 0;
    let mut start = 0;
    while start + r + 1 <= id {
        start += r + 1;
        r += 1;
    }
    (r, id - start)
}

pub fn stage_id(row: usize, col: usize) -> usize {
    row * (row + 1) / 2 + col
}

pub struct Stage {
    pub id: usize,
    pub row: usize,
    pub col: usize,
    pub segs: Vec<Seg>,
    pub has_fork: bool,
}

pub const FORK_START: usize = STAGE_SEGS - FORK_SEGS;

impl Stage {
    pub fn len_m(&self) -> f32 {
        STAGE_SEGS as f32 * SEG_LEN
    }
    pub fn split(&self, i: f32) -> f32 {
        if !self.has_fork || i < FORK_START as f32 {
            return 0.0;
        }
        let t = (i - FORK_START as f32) / FORK_SEGS as f32;
        FORK_SPLIT * t * t
    }
    pub fn split_at_z(&self, z: f32) -> f32 {
        self.split(z / SEG_LEN)
    }
    pub fn seg_index(&self, z: f32) -> usize {
        ((z / SEG_LEN).max(0.0) as usize).min(self.segs.len() - 2)
    }
    pub fn ground_y(&self, z: f32) -> f32 {
        let i = self.seg_index(z);
        let f = (z / SEG_LEN - i as f32).clamp(0.0, 1.0);
        self.segs[i].y + (self.segs[i + 1].y - self.segs[i].y) * f
    }
    pub fn curve_at(&self, z: f32) -> f32 {
        self.segs[self.seg_index(z)].curve
    }
    /// Road centres at a given z (one, or two inside the fork).
    pub fn centers(&self, z: f32) -> [Option<f32>; 2] {
        let s = self.split_at_z(z);
        if s > 0.0 { [Some(-s), Some(s)] } else { [Some(0.0), None] }
    }
    pub fn on_road(&self, z: f32, x: f32) -> bool {
        self.centers(z).iter().flatten().any(|c| (x - c).abs() <= ROAD_HALF + RUMBLE_W * 0.5)
    }

    pub fn generate(id: usize, themes: &[Theme]) -> Stage {
        let (row, col) = stage_row(id);
        let th = &themes[id];
        let has_fork = row + 1 < ROWS;
        let total = STAGE_SEGS + TAIL_SEGS + 2;
        let mut rng = Rng::new(id as u64 * 7919 + 1301);
        let mut segs: Vec<Seg> = (0..total).map(|_| Seg { curve: 0.0, y: 0.0, props: Vec::new() }).collect();

        // --- curves and hills ---
        let body_end = if has_fork { FORK_START - 30 } else { STAGE_SEGS - 40 };
        let mut i = 40;
        let mut y = 0.0f32;
        let mut ys = vec![0.0f32; total];
        while i < body_end {
            let len = (rng.range(45.0, 130.0) as usize).min(body_end - i);
            let straight = rng.f() < 0.28;
            let strength = if straight { 0.0 } else {
                let s = rng.range(0.35, 1.0) * th.curvy;
                (if rng.f() < 0.5 { -s } else { s }) * CURVE_HARD
            };
            let dh = rng.range(-1.0, 1.0) * th.hilly * 28.0;
            let dh = if (y + dh).abs() > 60.0 { -dh } else { dh };
            let y0 = y;
            for k in 0..len {
                let t = k as f32 / len as f32;
                let ease = if t < 0.25 { t / 0.25 } else if t > 0.75 { (1.0 - t) / 0.25 } else { 1.0 };
                segs[i + k].curve = strength * ease;
                ys[i + k] = y0 + dh * (1.0 - (t * std::f32::consts::PI).cos()) * 0.5;
            }
            y = y0 + dh;
            i += len;
        }
        // ease back to flat ground before the fork / goal
        let flat_start = body_end;
        let flat_len = 30usize;
        for k in 0..total - flat_start {
            let t = (k as f32 / flat_len as f32).min(1.0);
            ys[flat_start + k] = y * (1.0 + (t * std::f32::consts::PI).cos()) * 0.5;
        }
        for (s, yy) in segs.iter_mut().zip(ys.iter()) {
            s.y = *yy;
        }

        // --- roadside props ---
        let props = th.props;
        for i in 12..(STAGE_SEGS + TAIL_SEGS) {
            let fi = i as f32;
            let split = if has_fork && i >= FORK_START {
                let t = (fi - FORK_START as f32) / FORK_SEGS as f32;
                FORK_SPLIT * t * t
            } else { 0.0 };
            for side in [-1.0f32, 1.0] {
                if rng.f() < 0.22 {
                    let mut kind = props[rng.int(props.len())];
                    if kind == Prop::Lighthouse && rng.f() < 0.85 { kind = Prop::Rock; }
                    if kind == Prop::Billboard && rng.f() < 0.6 { kind = props[0]; }
                    let base = match kind {
                        Prop::Building => 15.0,
                        Prop::Billboard => 12.0,
                        Prop::Turbine => 22.0,
                        Prop::Lighthouse => 22.0,
                        Prop::Lamp => 0.8,
                        Prop::Pole => 2.5,
                        _ => 3.0,
                    };
                    let extra = match kind {
                        Prop::Lamp => 0.3,
                        Prop::Building => rng.range(0.0, 10.0),
                        _ => rng.range(0.0, 14.0),
                    };
                    let x = side * (split + ROAD_HALF + RUMBLE_W + base + extra);
                    segs[i].props.push(PropInst { kind, x, var: rng.f() });
                }
            }
            // regular lamps in towns
            if th.props.contains(&Prop::Lamp) && i % 14 == 0 {
                for side in [-1.0f32, 1.0] {
                    segs[i].props.push(PropInst { kind: Prop::Lamp, x: side * (split + ROAD_HALF + RUMBLE_W + 1.0), var: 0.5 });
                }
            }
            // dividers between the two branches once they separate
            if has_fork && i >= FORK_START && split > ROAD_HALF + 3.0 && i % 6 == 0 {
                segs[i].props.push(PropInst { kind: Prop::Divider, x: 0.0, var: 0.0 });
            }
        }
        if id == 0 {
            segs[24].props.push(PropInst { kind: Prop::StartGate, x: 0.0, var: 0.0 });
        }
        if has_fork {
            segs[FORK_START - 24].props.push(PropInst { kind: Prop::ForkSign, x: 0.0, var: 0.0 });
            for side in [-1.0f32, 1.0] {
                segs[STAGE_SEGS].props.push(PropInst { kind: Prop::Checkpoint, x: side * FORK_SPLIT, var: 0.0 });
            }
        } else {
            segs[STAGE_SEGS].props.push(PropInst { kind: Prop::GoalGate, x: 0.0, var: 0.0 });
        }
        Stage { id, row, col, segs, has_fork }
    }
}
