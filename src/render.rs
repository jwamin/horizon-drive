//! Pseudo-3D road renderer, procedural scenery, traffic and the player's car.
use crate::consts::*;
use crate::game::*;
use crate::track::*;
use macroquad::models::{Mesh, Vertex, draw_mesh};
use macroquad::prelude::*;

pub struct View {
    pub w: f32,
    pub h: f32,
    pub hz: f32,
    pub f: f32,
}

pub fn view() -> View {
    let w = screen_width();
    let h = screen_height();
    let f = (h * 0.5) / (FOV_V_DEG.to_radians() * 0.5).tan();
    View { w, h, hz: h * HORIZON_FRAC, f }
}

pub fn grad_rect(x: f32, y: f32, w: f32, h: f32, top: Color, bot: Color) {
    let v = |px: f32, py: f32, c: Color| Vertex::new(px, py, 0.0, 0.0, 0.0, c);
    let mesh = Mesh {
        vertices: vec![v(x, y, top), v(x + w, y, top), v(x + w, y + h, bot), v(x, y + h, bot)],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    };
    draw_mesh(&mesh);
}

pub fn quad(x1l: f32, x1r: f32, y1: f32, x2l: f32, x2r: f32, y2: f32, c: Color) {
    draw_triangle(vec2(x1l, y1), vec2(x1r, y1), vec2(x2r, y2), c);
    draw_triangle(vec2(x1l, y1), vec2(x2r, y2), vec2(x2l, y2), c);
}

#[derive(Clone, Copy)]
struct P {
    xa: f32,
    y: f32,
    dz: f32,
    sc: f32,
    sy: f32,
}

/// Fog painter: pulls colours toward the haze colour with distance.
#[derive(Clone, Copy)]
pub struct Fog {
    pub col: Color,
    pub t: f32,
}
impl Fog {
    pub fn c(&self, c: Color) -> Color {
        let mut o = lerp_color(c, self.col, self.t);
        o.a = c.a;
        o
    }
}

struct Pal {
    sky_top: Color,
    sky_bot: Color,
    far: Color,
    near_hill: Color,
    ground: [Color; 2],
    rumble: [Color; 2],
    road: [Color; 2],
    lane: Color,
    fog: Color,
    sun: Color,
    sea: Option<Color>,
    night: bool,
}

fn palette(a: &Theme, b: &Theme, t: f32) -> Pal {
    let l = |x: Color, y: Color| lerp_color(x, y, t);
    Pal {
        sky_top: l(a.sky_top, b.sky_top),
        sky_bot: l(a.sky_bot, b.sky_bot),
        far: l(a.far, b.far),
        near_hill: l(a.near_hill, b.near_hill),
        ground: [l(a.ground[0], b.ground[0]), l(a.ground[1], b.ground[1])],
        rumble: [l(a.rumble[0], b.rumble[0]), l(a.rumble[1], b.rumble[1])],
        road: [l(a.road[0], b.road[0]), l(a.road[1], b.road[1])],
        lane: l(a.lane, b.lane),
        fog: l(a.fog, b.fog),
        sun: l(a.sun, b.sun),
        sea: if t < 0.5 { a.sea.map(|s| l(s, b.sea.unwrap_or(s))) } else { b.sea.map(|s| l(a.sea.unwrap_or(s), s)) },
        night: if t < 0.5 { a.night } else { b.night },
    }
}

fn hash(n: i32) -> f32 {
    let mut x = (n as u32).wrapping_mul(374761393) ^ 0x9E3779B9;
    x = (x ^ (x >> 13)).wrapping_mul(1274126177);
    ((x ^ (x >> 16)) % 10000) as f32 / 10000.0
}

fn smooth_noise(u: f32) -> f32 {
    let i = u.floor() as i32;
    let f = u - u.floor();
    let t = f * f * (3.0 - 2.0 * f);
    hash(i) * (1.0 - t) + hash(i + 1) * t
}

fn draw_backdrop(r: &Race, v: &View, pal: &Pal, shake_y: f32) {
    let hz = v.hz + shake_y;
    grad_rect(0.0, 0.0, v.w, hz + 2.0, pal.sky_top, pal.sky_bot);
    let off = r.bg_offset;
    // sun / moon and stars
    let low_sun = pal.sky_bot.r > pal.sky_bot.b + 0.05;
    if pal.night {
        for i in 0..90 {
            let x = (hash(i * 3) + off * 0.2).rem_euclid(1.0) * v.w;
            let y = hash(i * 3 + 1) * hz * 0.85;
            let a = 0.4 + 0.6 * hash(i * 3 + 2);
            draw_rectangle(x, y, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, a));
        }
        draw_circle((0.7 - off * 0.3).rem_euclid(1.4) * v.w, hz * 0.3, v.h * 0.04, pal.sun);
    } else {
        let sx = (0.62 - off * 0.3).rem_euclid(1.4) * v.w - v.w * 0.2;
        let sy = if low_sun { hz - v.h * 0.07 } else { hz * 0.28 };
        let rad = if low_sun { v.h * 0.09 } else { v.h * 0.05 };
        for k in 0..4 {
            let mut c = pal.sun;
            c.a = 0.12;
            draw_circle(sx, sy, rad * (1.6 + k as f32 * 0.5), c);
        }
        draw_circle(sx, sy, rad, pal.sun);
        // clouds
        for i in 0..7 {
            let cx = (hash(100 + i) * 1.4 - off * 0.6).rem_euclid(1.4) * v.w - v.w * 0.2;
            let cy = hz * (0.12 + hash(200 + i) * 0.45);
            let s = v.h * (0.025 + hash(300 + i) * 0.03);
            let cc = Color::new(1.0, 1.0, 1.0, 0.85);
            draw_circle(cx, cy, s, cc);
            draw_circle(cx + s * 1.1, cy + s * 0.2, s * 0.85, cc);
            draw_circle(cx - s * 1.1, cy + s * 0.25, s * 0.75, cc);
            draw_rectangle(cx - s * 1.8, cy + s * 0.2, s * 3.6, s * 0.75, cc);
        }
    }
    // far mountains
    let step = 6.0;
    let n = (v.w / step) as i32 + 2;
    for i in 0..n {
        let x = i as f32 * step;
        let u = x / v.w * 3.0 + off * 1.0;
        let hgt = (smooth_noise(u) * 0.7 + smooth_noise(u * 2.7 + 9.0) * 0.3) * v.h * 0.13 + v.h * 0.02;
        draw_rectangle(x, hz - hgt, step + 1.0, hgt + 2.0, pal.far);
    }
    // nearer hills or skyline
    for i in 0..n {
        let x = i as f32 * step;
        let u = x / v.w * 5.0 + off * 2.0;
        if pal.night {
            let b = (u * 1.5).floor();
            let hgt = (0.03 + hash(b as i32 + 500) * 0.12) * v.h;
            draw_rectangle(x, hz - hgt, step + 1.0, hgt + 2.0, pal.near_hill);
            if hash((u * 12.0).floor() as i32 * 7 + (b as i32)) > 0.75 {
                let wy = hz - hgt * hash((x as i32) / 6 + 77);
                draw_rectangle(x + 1.0, wy, 2.0, 2.0, Color::new(1.0, 0.85, 0.4, 0.9));
            }
        } else {
            let hgt = (smooth_noise(u + 40.0) * 0.6 + 0.1) * v.h * 0.06;
            draw_rectangle(x, hz - hgt, step + 1.0, hgt + 2.0, pal.near_hill);
        }
    }
    // ground beyond the draw distance
    let far_ground = pal.sea.unwrap_or(lerp_color(pal.ground[0], pal.fog, 0.6));
    draw_rectangle(0.0, hz, v.w, v.h - hz, far_ground);
}

pub fn draw_world(r: &Race, themes: &[Theme]) {
    let v = view();
    let cur = &themes[r.stage.id];
    let prev = &themes[r.prev_theme];
    let tb = (r.z / 400.0).clamp(0.0, 1.0);
    let pal = palette(prev, cur, tb);
    let shake_x = (r.clock * 37.0).sin() * r.shake * 2.0;
    let shake_y = (r.clock * 53.0).cos() * r.shake * 2.0;
    draw_backdrop(r, &v, &pal, shake_y);
    let hz = v.hz + shake_y;
    let cx0 = v.w * 0.5 + shake_x;

    let cam_z = r.z - CAM_BACK;
    let cam_y = r.ground_y() + CAM_HEIGHT;
    let base = (cam_z / SEG_LEN).floor() as isize;
    let frac = cam_z / SEG_LEN - base as f32;
    let segs = &r.stage.segs;
    let curve = |i: isize| -> f32 { if i < 0 || i as usize >= segs.len() { 0.0 } else { segs[i as usize].curve } };
    let yat = |i: isize| -> f32 { if i < 0 { segs[0].y } else { segs[(i as usize).min(segs.len() - 1)].y } };
    let max_n = DRAW_SEGS.min((segs.len() as isize - base - 1).max(1) as usize);

    let mut pts: Vec<P> = Vec::with_capacity(max_n + 1);
    let mut x = 0.0f32;
    let mut dx = -(curve(base) * frac);
    for n in 0..=max_n {
        let i = base + n as isize;
        let dz = (i as f32 * SEG_LEN - cam_z).max(0.3);
        let y = yat(i);
        let sc = v.f / dz;
        pts.push(P { xa: x, y, dz, sc, sy: hz - (y - cam_y) * sc });
        x += dx;
        dx += curve(i);
    }
    let sxp = |p: &P, wx: f32| cx0 + (p.xa + wx - r.cam_x) * p.sc;

    // bucket traffic and balloons per segment
    let mut car_b: Vec<Vec<usize>> = vec![Vec::new(); max_n];
    for (k, t) in r.traffic.iter().enumerate() {
        let n = (t.z / SEG_LEN).floor() as isize - base;
        if n >= 0 && (n as usize) < max_n && t.z - cam_z > 1.0 {
            car_b[n as usize].push(k);
        }
    }
    let mut bal_b: Vec<Vec<usize>> = vec![Vec::new(); max_n];
    for (k, b) in r.balloons.iter().enumerate() {
        let n = (b.0 / SEG_LEN).floor() as isize - base;
        if !b.2 && n >= 0 && (n as usize) < max_n && b.0 - cam_z > 1.0 {
            bal_b[n as usize].push(k);
        }
    }
    let player_n = ((r.z / SEG_LEN).floor() as isize - base).clamp(0, max_n as isize - 1) as usize;
    let draw_dist = DRAW_SEGS as f32 * SEG_LEN;
    let sea_side = if r.stage.id % 2 == 0 { -1.0 } else { 1.0 };

    let interp = |z: f32| -> P {
        let nf = z / SEG_LEN - base as f32;
        let n = (nf.floor().max(0.0) as usize).min(max_n - 1);
        let f = (nf - n as f32).clamp(0.0, 1.0);
        let a = pts[n];
        let b = pts[n + 1];
        let dz = (z - cam_z).max(0.3);
        let y = a.y + (b.y - a.y) * f;
        let sc = v.f / dz;
        P { xa: a.xa + (b.xa - a.xa) * f, y, dz, sc, sy: hz - (y - cam_y) * sc }
    };

    for n in (0..max_n).rev() {
        let i = base + n as isize;
        let near = pts[n];
        let far = pts[n + 1];
        if far.dz <= 0.31 {
            continue;
        }
        let fog = Fog { col: pal.fog, t: ((far.dz / draw_dist).clamp(0.0, 1.0)).powf(1.5) * 0.85 };
        let stripe = ((i.max(0) / 3) % 2) as usize;
        let visible = far.sy < near.sy;
        if visible {
            draw_rectangle(0.0, far.sy, v.w, near.sy - far.sy + 1.0, fog.c(pal.ground[stripe]));
            let fi = i as f32;
            let s1 = r.stage.split(fi);
            let s2 = r.stage.split(fi + 1.0);
            if let Some(sea) = pal.sea {
                let edge = ROAD_HALF + 26.0;
                let a = sxp(&near, sea_side * (s1 + edge));
                let b = sxp(&far, sea_side * (s2 + edge));
                let sea_c = fog.c(sea);
                let sand = fog.c(lerp_color(pal.ground[stripe], WHITE, 0.25));
                let a2 = sxp(&near, sea_side * (s1 + edge - 3.0));
                let b2 = sxp(&far, sea_side * (s2 + edge - 3.0));
                if sea_side < 0.0 {
                    quad(a2.min(a), a2, near.sy, b2.min(b), b2, far.sy, sand);
                    quad(-20.0, a, near.sy, -20.0, b, far.sy, sea_c);
                } else {
                    quad(a2, a2.max(a), near.sy, b2, b2.max(b), far.sy, sand);
                    quad(a, v.w + 20.0, near.sy, b, v.w + 20.0, far.sy, sea_c);
                }
            }
            let branches: &[f32] = if s1 > 0.0 || s2 > 0.0 { &[-1.0, 1.0] } else { &[0.0] };
            for &side in branches {
                let c1 = side * s1;
                let c2 = side * s2;
                let rw = ROAD_HALF + RUMBLE_W;
                quad(sxp(&near, c1 - rw), sxp(&near, c1 + rw), near.sy, sxp(&far, c2 - rw), sxp(&far, c2 + rw), far.sy, fog.c(pal.rumble[stripe]));
            }
            for &side in branches {
                let c1 = side * s1;
                let c2 = side * s2;
                let rw = ROAD_HALF;
                quad(sxp(&near, c1 - rw), sxp(&near, c1 + rw), near.sy, sxp(&far, c2 - rw), sxp(&far, c2 + rw), far.sy, fog.c(pal.road[stripe]));
                let lane = fog.c(pal.lane);
                let lw = 0.09;
                for e in [-ROAD_HALF + 0.45, ROAD_HALF - 0.45] {
                    quad(sxp(&near, c1 + e - lw), sxp(&near, c1 + e + lw), near.sy, sxp(&far, c2 + e - lw), sxp(&far, c2 + e + lw), far.sy, lane);
                }
                if (i.max(0) / 2) % 2 == 0 {
                    for e in [-4.0f32, 0.0, 4.0] {
                        let lw = 0.12;
                        quad(sxp(&near, c1 + e - lw), sxp(&near, c1 + e + lw), near.sy, sxp(&far, c2 + e - lw), sxp(&far, c2 + e + lw), far.sy, lane);
                    }
                }
            }
        }
        // props on this segment
        if i >= 0 {
            let seg = &segs[i as usize];
            for p in &seg.props {
                let sx = sxp(&near, p.x);
                if sx < -v.w * 1.5 || sx > v.w * 2.5 || near.sc < 0.25 {
                    continue;
                }
                draw_prop(p, sx, near.sy, near.sc, fog, r, themes, pal.night);
            }
        }
        for &k in &bal_b[n] {
            let b = r.balloons[k];
            let p = interp(b.0);
            let sx = cx0 + (p.xa + b.1 - r.cam_x) * p.sc;
            draw_balloon(sx, p.sy, p.sc, fog, k, r.clock);
        }
        let mut cars: Vec<usize> = car_b[n].clone();
        cars.sort_by(|a, b| r.traffic[*b].z.partial_cmp(&r.traffic[*a].z).unwrap());
        for k in cars {
            let t = &r.traffic[k];
            let p = interp(t.z);
            let sx = cx0 + (p.xa + t.x - r.cam_x) * p.sc;
            draw_traffic(t, sx, p.sy, p.sc, fog, pal.night);
        }
        if n == player_n {
            let p = interp(r.z);
            let sx = cx0 + (p.xa + r.x - r.cam_x) * p.sc;
            let mut sy = p.sy;
            if r.offroad && r.speed > 20.0 {
                sy += (r.clock * 45.0).sin() * 2.5;
            }
            if r.crash_t > 0.0 {
                sy -= ((CRASH_TIME - r.crash_t) * 5.0).sin().abs() * 0.5 * p.sc * (r.crash_t / CRASH_TIME);
            }
            draw_particles(r, sx, p.sy, p.sc, &pal);
            draw_player_car(r, sx, sy, p.sc);
        }
    }
}

fn draw_particles(r: &Race, sx: f32, sy: f32, s: f32, pal: &Pal) {
    for p in &r.particles {
        let a = (p.life / 0.7).clamp(0.0, 1.0);
        let col = if p.dust {
            let mut c = lerp_color(pal.ground[0], BLACK, 0.2);
            c.a = a * 0.7;
            c
        } else {
            Color::new(0.95, 0.95, 0.95, a * 0.6)
        };
        let rad = (0.25 + (1.0 - a) * 0.9) * s;
        draw_circle(sx + p.sx * 2.0 * s, sy - p.sy * 1.5 * s - 0.2 * s, rad, col);
    }
}

fn rect_m(sx: f32, sy: f32, s: f32, x0: f32, y0: f32, x1: f32, y1: f32, c: Color) {
    draw_rectangle(sx + x0 * s, sy - y1 * s, (x1 - x0) * s, (y1 - y0) * s, c);
}

fn draw_balloon(sx: f32, sy: f32, s: f32, fog: Fog, k: usize, clock: f32) {
    let cols = [RED, ORANGE, Color::from_rgba(250, 80, 180, 255), Color::from_rgba(60, 170, 250, 255), YELLOW];
    let bob = (clock * 3.0 + k as f32).sin() * 0.15;
    let cy = sy - (2.0 + bob) * s;
    draw_line(sx, cy, sx, sy - 0.6 * s, (0.04 * s).max(1.0), fog.c(WHITE));
    draw_ellipse(sx, cy, 0.55 * s, 0.7 * s, 0.0, fog.c(cols[k % cols.len()]));
    draw_circle(sx - 0.18 * s, cy - 0.25 * s, 0.12 * s, Color::new(1.0, 1.0, 1.0, 0.5));
}

fn draw_traffic(t: &Traffic, sx: f32, sy: f32, s: f32, fog: Fog, night: bool) {
    let (w, h, _) = t.dims();
    let hw = w * 0.5;
    let dark = Color::new(0.08, 0.08, 0.1, 1.0);
    // shadow
    draw_ellipse(sx, sy, hw * 1.1 * s, 0.25 * s, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));
    // wheels
    rect_m(sx, sy, s, -hw + 0.1, 0.0, -hw + 0.45, 0.6, fog.c(dark));
    rect_m(sx, sy, s, hw - 0.45, 0.0, hw - 0.1, 0.6, fog.c(dark));
    let body = fog.c(t.color);
    let shade = fog.c(lerp_color(t.color, BLACK, 0.3));
    match t.kind {
        3 => {
            // truck: cargo box
            rect_m(sx, sy, s, -hw, 0.35, hw, h, fog.c(Color::from_rgba(225, 225, 220, 255)));
            rect_m(sx, sy, s, -hw, 0.35, hw, 0.75, shade);
            rect_m(sx, sy, s, -0.03, 0.8, 0.03, h - 0.1, fog.c(GRAY));
            rect_m(sx, sy, s, -hw + 0.1, h - 0.25, hw - 0.1, h - 0.15, body);
        }
        4 => {
            // bus
            rect_m(sx, sy, s, -hw, 0.3, hw, h, body);
            rect_m(sx, sy, s, -hw + 0.15, h * 0.55, hw - 0.15, h - 0.25, fog.c(Color::from_rgba(40, 50, 70, 255)));
            rect_m(sx, sy, s, -hw, 0.3, hw, 0.6, shade);
        }
        2 => {
            rect_m(sx, sy, s, -hw, 0.35, hw, h, body);
            rect_m(sx, sy, s, -hw + 0.2, h * 0.6, hw - 0.2, h - 0.2, fog.c(Color::from_rgba(40, 50, 70, 255)));
            rect_m(sx, sy, s, -hw, 0.35, hw, 0.6, shade);
        }
        _ => {
            rect_m(sx, sy, s, -hw, 0.3, hw, h * 0.6, body);
            let tw = hw * 0.8;
            quad(sx - hw * 0.95 * s, sx + hw * 0.95 * s, sy - h * 0.6 * s, sx - tw * 0.8 * s, sx + tw * 0.8 * s, sy - h * s, body);
            quad(sx - hw * 0.75 * s, sx + hw * 0.75 * s, sy - h * 0.63 * s, sx - tw * 0.68 * s, sx + tw * 0.68 * s, sy - h * 0.94 * s, fog.c(Color::from_rgba(40, 50, 70, 255)));
            rect_m(sx, sy, s, -hw, 0.3, hw, 0.5, shade);
        }
    }
    // tail lights
    let tl = if night { Color::from_rgba(255, 60, 60, 255) } else { fog.c(Color::from_rgba(200, 20, 20, 255)) };
    let ly = if t.kind >= 2 { 0.7 } else { 0.62 };
    rect_m(sx, sy, s, -hw + 0.05, ly, -hw + 0.4, ly + 0.15, tl);
    rect_m(sx, sy, s, hw - 0.4, ly, hw - 0.05, ly + 0.15, tl);
    if night {
        draw_circle(sx - (hw - 0.22) * s, sy - (ly + 0.07) * s, 0.35 * s, Color::new(1.0, 0.2, 0.2, 0.25));
        draw_circle(sx + (hw - 0.22) * s, sy - (ly + 0.07) * s, 0.35 * s, Color::new(1.0, 0.2, 0.2, 0.25));
    }
    rect_m(sx, sy, s, -0.25, 0.4, 0.25, 0.55, fog.c(Color::from_rgba(235, 235, 220, 255)));
}

pub fn draw_player_car(r: &Race, sx: f32, sy: f32, s: f32) {
    let spec = r.spec();
    let body = Color::from_rgba(spec.body.0, spec.body.1, spec.body.2, 255);
    draw_car_shape(sx, sy, s, r.drift_angle, body, r.braking, r.steer, r.clock, r.speed);
}

pub fn draw_car_shape(sx: f32, sy: f32, s: f32, ang: f32, body: Color, braking: bool, steer: f32, clock: f32, speed: f32) {
    let cw = ang.cos().abs().max(0.35);
    let sw = ang.sin();
    let hw = 0.98 * cw;
    let dark = Color::new(0.07, 0.07, 0.08, 1.0);
    let shade = lerp_color(body, BLACK, 0.35);
    let side_c = lerp_color(body, BLACK, 0.2);
    let lean = steer * 0.06;
    // shadow
    draw_ellipse(sx, sy, (hw + sw.abs() * 1.6) * 1.1 * s, 0.28 * s, 0.0, Color::new(0.0, 0.0, 0.0, 0.35));
    // side panel visible when the car is angled
    if sw.abs() > 0.1 {
        let sd = -sw.signum();
        let ext = sw.abs() * 2.0;
        let rise = ext * 0.18;
        let x0 = sx + sd * (hw - 0.05) * s;
        let x1 = sx + sd * (hw + ext) * s;
        let (xl, xr) = if sd < 0.0 { (x1, x0) } else { (x0, x1) };
        let (ryl, ryr) = if sd < 0.0 { (rise, 0.0) } else { (0.0, rise) };
        let yb = |r: f32| sy - (0.25 + r) * s;
        let yt = |r: f32| sy - (0.88 + r) * s;
        draw_triangle(vec2(xl, yb(ryl)), vec2(xr, yb(ryr)), vec2(xr, yt(ryr)), side_c);
        draw_triangle(vec2(xl, yb(ryl)), vec2(xr, yt(ryr)), vec2(xl, yt(ryl)), side_c);
        // front wheel peeking out under the side
        let wx = sx + sd * (hw + ext * 0.8) * s;
        let ww = (0.15 + ext * 0.2) * s;
        draw_rectangle(wx - ww * 0.5, sy - (0.5 + rise * 0.8) * s, ww, 0.5 * s, dark);
    }
    // ---- 1990s mid-engine barchetta, seen from behind (original drawing, no badges) ----
    let l1 = lean;
    let l2 = lean * 1.6;
    let l3 = lean * 2.2;
    let deck = lerp_color(body, WHITE, 0.12);
    let grille = Color::from_rgba(22, 22, 26, 255);
    let mesh = Color::from_rgba(55, 55, 62, 255);
    // wide rear tyres
    rect_m(sx, sy, s, -hw - 0.02, 0.0, -hw + 0.5, 0.62, dark);
    rect_m(sx, sy, s, hw - 0.5, 0.0, hw + 0.02, 0.62, dark);
    // lower body: wide hips
    quad(sx - (hw + 0.04) * s, sx + (hw + 0.04) * s, sy - 0.2 * s, sx - (hw + 0.06 - l1) * s, sx + (hw + 0.06 + l1) * s, sy - 0.66 * s, body);
    // diffuser with fins
    rect_m(sx, sy, s, -hw + 0.2, 0.12, hw - 0.2, 0.25, grille);
    for k in 0..7 {
        let fx = -0.6 * cw + k as f32 * 0.2 * cw;
        rect_m(sx, sy, s, fx - 0.015, 0.12, fx + 0.015, 0.25, mesh);
    }
    // black mesh tail panel spanning the rear
    rect_m(sx, sy, s, -hw + 0.12 + l1, 0.3, hw - 0.12 + l1, 0.62, grille);
    for k in 0..5 {
        let y = 0.34 + k as f32 * 0.055;
        rect_m(sx, sy, s, -hw + 0.14 + l1, y, hw - 0.14 + l1, y + 0.012, mesh);
    }
    // twin round tail lights each side, set into the mesh
    let tl = if braking { Color::from_rgba(255, 60, 50, 255) } else { Color::from_rgba(185, 18, 22, 255) };
    for side in [-1.0f32, 1.0] {
        for off in [0.3f32, 0.62] {
            let cx = sx + (side * (hw - off) * 1.0 + l1) * s;
            let cy = sy - 0.5 * s;
            draw_circle(cx, cy, 0.12 * s, Color::from_rgba(10, 10, 12, 255));
            draw_circle(cx, cy, 0.095 * s, tl);
            draw_circle(cx - 0.03 * s, cy - 0.03 * s, 0.03 * s, Color::new(1.0, 0.8, 0.8, 0.5));
            if braking {
                draw_circle(cx, cy, 0.3 * s, Color::new(1.0, 0.2, 0.15, 0.18));
            }
        }
    }
    // centre exhausts and a small plate
    for ex in [-0.11f32, 0.11] {
        draw_circle(sx + (ex * cw + l1) * s, sy - 0.4 * s, 0.075 * s, Color::from_rgba(160, 160, 165, 255));
        draw_circle(sx + (ex * cw + l1) * s, sy - 0.4 * s, 0.05 * s, Color::from_rgba(15, 15, 15, 255));
    }
    if speed > 5.0 && (clock * 30.0).sin() > 0.6 {
        draw_circle(sx + l1 * s, sy - 0.36 * s, 0.14 * s, Color::new(0.8, 0.8, 0.8, 0.22));
    }
    rect_m(sx, sy, s, -0.22 * cw + l1, 0.26, 0.22 * cw + l1, 0.33, Color::from_rgba(240, 240, 230, 255));
    // body band above the tail panel
    quad(sx - (hw + 0.06 - l1) * s, sx + (hw + 0.06 + l1) * s, sy - 0.66 * s, sx - (hw - 0.02 - l2) * s, sx + (hw - 0.02 + l2) * s, sy - 0.8 * s, body);
    // windscreen frame (far away, low and raked)
    let wl = sx - (hw - 0.3 - l3) * s;
    let wr = sx + (hw - 0.3 + l3) * s;
    draw_triangle(vec2(wl, sy - 0.95 * s), vec2(wr, sy - 0.95 * s), vec2(wr - 0.16 * s, sy - 1.22 * s), Color::new(0.6, 0.8, 0.9, 0.22));
    draw_triangle(vec2(wl, sy - 0.95 * s), vec2(wr - 0.16 * s, sy - 1.22 * s), vec2(wl + 0.16 * s, sy - 1.22 * s), Color::new(0.6, 0.8, 0.9, 0.22));
    draw_line(wl + 0.16 * s, sy - 1.22 * s, wr - 0.16 * s, sy - 1.22 * s, 0.05 * s, dark);
    // occupants (open top)
    for (i, px) in [-0.4f32, 0.4].iter().enumerate() {
        let x = sx + (px * cw + l3) * s;
        let shirt = if i == 0 { Color::from_rgba(40, 120, 190, 255) } else { Color::from_rgba(240, 120, 150, 255) };
        draw_rectangle(x - 0.25 * s, sy - 1.08 * s, 0.5 * s, 0.16 * s, shirt);
        let hair = if i == 0 { Color::from_rgba(50, 35, 25, 255) } else { Color::from_rgba(150, 70, 40, 255) };
        draw_rectangle(x - 0.05 * s, sy - 1.16 * s, 0.1 * s, 0.1 * s, Color::from_rgba(220, 170, 130, 255));
        draw_circle(x, sy - 1.26 * s, 0.13 * s, hair);
        if i == 1 {
            draw_circle(x + 0.1 * s, sy - 1.17 * s, 0.075 * s, hair);
            draw_circle(x - 0.1 * s, sy - 1.17 * s, 0.075 * s, hair);
        }
        // roll-hoop fairing behind each seat
        let hx = sx + (px * cw + l2) * s;
        draw_ellipse(hx, sy - 0.98 * s, 0.3 * s, 0.13 * s, 0.0, deck);
    }
    // engine cover with louvres between the wing end plates
    quad(sx - (hw - 0.08 - l2) * s, sx + (hw - 0.08 + l2) * s, sy - 0.8 * s, sx - (hw - 0.2 - l3) * s, sx + (hw - 0.2 + l3) * s, sy - 0.92 * s, deck);
    for k in 0..6 {
        let fx = -0.5 * cw + k as f32 * 0.2 * cw + lean * 1.9;
        rect_m(sx, sy, s, fx - 0.05, 0.83, fx + 0.05, 0.89, grille);
    }
    // the rear wing: full-width blade on swept end plates rising from the hips
    let wy0 = 0.98;
    let wy1 = 1.06;
    for side in [-1.0f32, 1.0] {
        let bx = sx + (side * (hw - 0.04) + l1) * s;
        let tx = sx + (side * (hw - 0.02) + l2) * s;
        let ins = side * 0.1 * s;
        draw_triangle(vec2(bx, sy - 0.74 * s), vec2(bx - ins, sy - 0.74 * s), vec2(tx - ins, sy - wy1 * s), body);
        draw_triangle(vec2(bx, sy - 0.74 * s), vec2(tx - ins, sy - wy1 * s), vec2(tx, sy - wy1 * s), body);
    }
    quad(sx - (hw + 0.02 - l2) * s, sx + (hw + 0.02 + l2) * s, sy - wy0 * s, sx - (hw + 0.02 - l2) * s, sx + (hw + 0.02 + l2) * s, sy - wy1 * s, body);
    quad(sx - (hw - 0.0 - l2) * s, sx + (hw - 0.0 + l2) * s, sy - wy0 * s, sx - (hw - 0.0 - l2) * s, sx + (hw - 0.0 + l2) * s, sy - (wy0 + 0.02) * s, shade);
    quad(sx - (hw + 0.02 - l2) * s, sx + (hw + 0.02 + l2) * s, sy - (wy1 - 0.015) * s, sx - (hw + 0.02 - l2) * s, sx + (hw + 0.02 + l2) * s, sy - wy1 * s, lerp_color(body, WHITE, 0.3));
}

fn gantry(sx: f32, sy: f32, s: f32, fog: Fog, banner: Color, text: &str, text2: Option<&str>) {
    let post = fog.c(Color::from_rgba(70, 70, 80, 255));
    let hw = ROAD_HALF + 1.4;
    rect_m(sx, sy, s, -hw - 0.3, 0.0, -hw + 0.3, 7.6, post);
    rect_m(sx, sy, s, hw - 0.3, 0.0, hw + 0.3, 7.6, post);
    rect_m(sx, sy, s, -hw, 5.6, hw, 7.6, fog.c(banner));
    rect_m(sx, sy, s, -hw, 5.6, hw, 5.8, fog.c(WHITE));
    rect_m(sx, sy, s, -hw, 7.4, hw, 7.6, fog.c(WHITE));
    let fs = (1.4 * s).min(160.0);
    if fs >= 6.0 {
        if let Some(t2) = text2 {
            let fs2 = fs * 0.8;
            let d = measure_text(text, None, fs2 as u16, 1.0);
            draw_text(text, sx - hw * s * 0.5 - d.width * 0.5, sy - 6.25 * s, fs2, fog.c(WHITE));
            let d = measure_text(t2, None, fs2 as u16, 1.0);
            draw_text(t2, sx + hw * s * 0.5 - d.width * 0.5, sy - 6.25 * s, fs2, fog.c(WHITE));
        } else {
            let d = measure_text(text, None, fs as u16, 1.0);
            draw_text(text, sx - d.width * 0.5, sy - 6.2 * s, fs, fog.c(WHITE));
        }
    }
}

fn draw_prop(p: &PropInst, sx: f32, sy: f32, s: f32, fog: Fog, r: &Race, themes: &[Theme], night: bool) {
    let v = p.var;
    let trunk = fog.c(Color::from_rgba(120, 85, 55, 255));
    match p.kind {
        Prop::Palm => {
            let h = 8.0 + v * 4.0;
            let lean = if p.x < 0.0 { 1.2 } else { -1.2 } * (0.5 + v);
            let segs = 6;
            for k in 0..segs {
                let t0 = k as f32 / segs as f32;
                let t1 = (k + 1) as f32 / segs as f32;
                let x0 = sx + lean * t0 * t0 * s;
                let x1 = sx + lean * t1 * t1 * s;
                let w = (0.45 - 0.2 * t0) * s;
                quad(x0 - w * 0.5, x0 + w * 0.5, sy - h * t0 * s, x1 - w * 0.4, x1 + w * 0.4, sy - h * t1 * s, trunk);
            }
            let tx = sx + lean * s;
            let ty = sy - h * s;
            let leaf = fog.c(Color::from_rgba(40, 140, 60, 255));
            let leaf2 = fog.c(Color::from_rgba(30, 110, 50, 255));
            for k in 0..7 {
                let a = k as f32 / 7.0 * std::f32::consts::TAU + v * 3.0;
                let len = 3.4 * s;
                let ex = tx + a.cos() * len;
                let ey = ty + a.sin().abs() * len * 0.35 + len * 0.35;
                let mx = tx + a.cos() * len * 0.5;
                let my = ty - len * 0.12;
                let c = if k % 2 == 0 { leaf } else { leaf2 };
                draw_triangle(vec2(tx, ty - 0.2 * s), vec2(mx, my - 0.35 * s), vec2(ex, ey), c);
                draw_triangle(vec2(tx, ty + 0.2 * s), vec2(mx, my + 0.25 * s), vec2(ex, ey), c);
            }
            draw_circle(tx, ty + 0.3 * s, 0.35 * s, fog.c(Color::from_rgba(90, 60, 30, 255)));
        }
        Prop::Pine | Prop::SnowPine => {
            let h = 9.0 + v * 6.0;
            rect_m(sx, sy, s, -0.35, 0.0, 0.35, h * 0.3, trunk);
            let g = fog.c(Color::from_rgba(25, 85, 45, 255));
            let g2 = fog.c(Color::from_rgba(35, 105, 55, 255));
            for k in 0..3 {
                let t = k as f32;
                let yb = h * (0.2 + t * 0.24);
                let yt = yb + h * 0.42;
                let w = 2.6 - t * 0.6;
                draw_triangle(vec2(sx - w * s, sy - yb * s), vec2(sx + w * s, sy - yb * s), vec2(sx, sy - yt * s), if k % 2 == 0 { g } else { g2 });
                if p.kind == Prop::SnowPine {
                    let sn = fog.c(WHITE);
                    draw_triangle(vec2(sx - w * 0.35 * s, sy - (yt - h * 0.15) * s), vec2(sx + w * 0.35 * s, sy - (yt - h * 0.15) * s), vec2(sx, sy - yt * s), sn);
                }
            }
        }
        Prop::Cypress => {
            let h = 8.0 + v * 4.0;
            rect_m(sx, sy, s, -0.2, 0.0, 0.2, 1.0, trunk);
            draw_ellipse(sx, sy - h * 0.55 * s, 1.1 * s, h * 0.48 * s, 0.0, fog.c(Color::from_rgba(30, 80, 40, 255)));
            draw_ellipse(sx - 0.3 * s, sy - h * 0.6 * s, 0.5 * s, h * 0.35 * s, 0.0, fog.c(Color::from_rgba(45, 100, 50, 255)));
        }
        Prop::Cactus => {
            let g = fog.c(Color::from_rgba(70, 140, 70, 255));
            let h = 4.0 + v * 3.0;
            rect_m(sx, sy, s, -0.4, 0.0, 0.4, h, g);
            draw_circle(sx, sy - h * s, 0.4 * s, g);
            rect_m(sx, sy, s, -1.4, h * 0.45, -0.4, h * 0.55, g);
            rect_m(sx, sy, s, -1.4, h * 0.45, -1.0, h * 0.8, g);
            rect_m(sx, sy, s, 0.4, h * 0.35, 1.3, h * 0.45, g);
            rect_m(sx, sy, s, 0.9, h * 0.35, 1.3, h * 0.7, g);
        }
        Prop::Building => {
            let h = 9.0 + v * 22.0;
            let w = 11.0;
            let cols = [(210, 190, 160), (190, 120, 90), (230, 230, 220), (150, 170, 190), (200, 160, 120)];
            let cc = cols[(v * 4.99) as usize];
            let base = if night { Color::from_rgba(cc.0 / 4, cc.1 / 4, cc.2 / 3, 255) } else { Color::from_rgba(cc.0, cc.1, cc.2, 255) };
            rect_m(sx, sy, s, -w * 0.5, 0.0, w * 0.5, h, fog.c(base));
            rect_m(sx, sy, s, -w * 0.5, h - 0.6, w * 0.5, h, fog.c(lerp_color(base, BLACK, 0.3)));
            if s * 1.0 > 1.5 {
                let rows = (h / 3.0) as i32;
                for ry in 0..rows {
                    for cx in 0..4 {
                        let lit = night && hash(ry * 13 + cx * 7 + (v * 1000.0) as i32) > 0.45;
                        let wc = if lit { Color::from_rgba(255, 220, 120, 255) } else if night { Color::from_rgba(20, 20, 35, 255) } else { fog.c(Color::from_rgba(60, 90, 120, 255)) };
                        let x0 = -w * 0.5 + 0.9 + cx as f32 * 2.6;
                        let y0 = 1.5 + ry as f32 * 3.0;
                        rect_m(sx, sy, s, x0, y0, x0 + 1.5, y0 + 1.6, wc);
                    }
                }
            }
        }
        Prop::Rock => {
            let c = fog.c(Color::from_rgba(130, 120, 110, 255));
            let c2 = fog.c(Color::from_rgba(105, 98, 90, 255));
            let w = 2.0 + v * 2.0;
            draw_ellipse(sx, sy - w * 0.4 * s, w * s, w * 0.6 * s, 0.0, c);
            draw_ellipse(sx + w * 0.3 * s, sy - w * 0.3 * s, w * 0.6 * s, w * 0.4 * s, 0.0, c2);
        }
        Prop::Lamp => {
            let post = fog.c(Color::from_rgba(60, 60, 70, 255));
            let dir = if p.x < 0.0 { 1.0 } else { -1.0 };
            rect_m(sx, sy, s, -0.12, 0.0, 0.12, 8.0, post);
            let ax = sx + dir * 2.0 * s;
            draw_line(sx, sy - 8.0 * s, ax, sy - 8.0 * s, (0.18 * s).max(1.0), post);
            let lc = if night { Color::from_rgba(255, 230, 150, 255) } else { fog.c(Color::from_rgba(230, 230, 210, 255)) };
            draw_rectangle(ax - 0.5 * s, sy - 8.0 * s, s, 0.25 * s, lc);
            if night {
                draw_circle(ax, sy - 7.8 * s, 1.2 * s, Color::new(1.0, 0.9, 0.6, 0.2));
            }
        }
        Prop::Billboard => {
            let post = fog.c(Color::from_rgba(80, 80, 85, 255));
            rect_m(sx, sy, s, -3.0, 0.0, -2.6, 4.0, post);
            rect_m(sx, sy, s, 2.6, 0.0, 3.0, 4.0, post);
            let cols = [(240, 120, 40), (40, 150, 220), (230, 60, 110), (60, 180, 120)];
            let cc = cols[(v * 3.99) as usize];
            rect_m(sx, sy, s, -4.0, 3.5, 4.0, 7.5, fog.c(Color::from_rgba(cc.0, cc.1, cc.2, 255)));
            rect_m(sx, sy, s, -3.6, 5.8, 3.6, 6.6, fog.c(WHITE));
            rect_m(sx, sy, s, -3.6, 4.2, 0.5, 5.0, fog.c(Color::from_rgba(255, 255, 255, 200)));
            draw_circle(sx + 2.2 * s, sy - 4.6 * s, 0.7 * s, fog.c(YELLOW));
        }
        Prop::Column => {
            let c = fog.c(Color::from_rgba(230, 225, 205, 255));
            let c2 = fog.c(Color::from_rgba(200, 195, 175, 255));
            let h = 5.0 + v * 4.0;
            rect_m(sx, sy, s, -0.9, 0.0, 0.9, 0.6, c2);
            rect_m(sx, sy, s, -0.6, 0.6, 0.6, h, c);
            rect_m(sx, sy, s, -0.25, 0.6, -0.1, h, c2);
            rect_m(sx, sy, s, 0.15, 0.6, 0.3, h, c2);
            if v > 0.4 {
                rect_m(sx, sy, s, -1.0, h, 1.0, h + 0.6, c2);
            }
        }
        Prop::Turbine => {
            let c = fog.c(Color::from_rgba(245, 245, 245, 255));
            let h = 30.0;
            quad(sx - 0.8 * s, sx + 0.8 * s, sy, sx - 0.4 * s, sx + 0.4 * s, sy - h * s, c);
            let hx = sx;
            let hy = sy - h * s;
            let a0 = r.clock * 1.5 + v * 6.0;
            for k in 0..3 {
                let a = a0 + k as f32 * std::f32::consts::TAU / 3.0;
                let ex = hx + a.cos() * 13.0 * s;
                let ey = hy + a.sin() * 13.0 * s;
                let px = -a.sin() * 0.5 * s;
                let py = a.cos() * 0.5 * s;
                draw_triangle(vec2(hx + px, hy + py), vec2(hx - px, hy - py), vec2(ex, ey), c);
            }
            draw_circle(hx, hy, 0.7 * s, fog.c(Color::from_rgba(200, 200, 200, 255)));
        }
        Prop::Bush => {
            let g = fog.c(Color::from_rgba(60, 130, 60, 255));
            let g2 = fog.c(Color::from_rgba(80, 150, 70, 255));
            draw_ellipse(sx, sy - 0.8 * s, 2.0 * s, 1.0 * s, 0.0, g);
            draw_ellipse(sx - 0.8 * s, sy - 1.1 * s, 1.0 * s, 0.8 * s, 0.0, g2);
            draw_ellipse(sx + 0.9 * s, sy - 1.0 * s, 0.9 * s, 0.7 * s, 0.0, g2);
        }
        Prop::Flowers => {
            let cols = [(240, 50, 60), (250, 200, 40), (250, 120, 180), (250, 140, 40)];
            let cc = cols[(v * 3.99) as usize];
            let fc = fog.c(Color::from_rgba(cc.0, cc.1, cc.2, 255));
            rect_m(sx, sy, s, -5.0, 0.0, 5.0, 0.5, fog.c(Color::from_rgba(70, 140, 60, 255)));
            if s > 2.0 {
                for k in 0..10 {
                    draw_circle(sx + (-4.5 + k as f32) * s, sy - 0.6 * s, 0.35 * s, fc);
                }
            } else {
                rect_m(sx, sy, s, -5.0, 0.4, 5.0, 0.9, fc);
            }
        }
        Prop::Lighthouse => {
            let h = 20.0;
            for k in 0..5 {
                let y0 = k as f32 * h / 5.0;
                let y1 = y0 + h / 5.0;
                let w0 = 2.2 - y0 * 0.04;
                let w1 = 2.2 - y1 * 0.04;
                let c = if k % 2 == 0 { WHITE } else { Color::from_rgba(210, 40, 40, 255) };
                quad(sx - w0 * s, sx + w0 * s, sy - y0 * s, sx - w1 * s, sx + w1 * s, sy - y1 * s, fog.c(c));
            }
            rect_m(sx, sy, s, -1.2, h, 1.2, h + 2.0, fog.c(Color::from_rgba(250, 240, 160, 255)));
            draw_triangle(vec2(sx - 1.6 * s, sy - (h + 2.0) * s), vec2(sx + 1.6 * s, sy - (h + 2.0) * s), vec2(sx, sy - (h + 3.5) * s), fog.c(Color::from_rgba(60, 60, 70, 255)));
        }
        Prop::Pole => {
            let c = fog.c(Color::from_rgba(110, 80, 60, 255));
            rect_m(sx, sy, s, -0.15, 0.0, 0.15, 9.0, c);
            rect_m(sx, sy, s, -1.4, 8.0, 1.4, 8.25, c);
        }
        Prop::Divider => {
            rect_m(sx, sy, s, -0.08, 0.0, 0.08, 0.7, fog.c(GRAY));
            rect_m(sx, sy, s, -0.8, 0.7, 0.8, 1.6, fog.c(Color::from_rgba(250, 210, 30, 255)));
            let k = fog.c(BLACK);
            draw_triangle(vec2(sx - 0.6 * s, sy - 1.15 * s), vec2(sx - 0.2 * s, sy - 1.45 * s), vec2(sx - 0.2 * s, sy - 0.85 * s), k);
            draw_triangle(vec2(sx + 0.6 * s, sy - 1.15 * s), vec2(sx + 0.2 * s, sy - 1.45 * s), vec2(sx + 0.2 * s, sy - 0.85 * s), k);
        }
        Prop::ForkSign => {
            let (row, col) = (r.stage.row + 1, r.stage.col);
            let a = themes[stage_id(row, col)].name;
            let b = themes[stage_id(row, col + 1)].name;
            let la = format!("< {}", a);
            let lb = format!("{} >", b);
            gantry(sx, sy, s, fog, Color::from_rgba(20, 110, 60, 255), &la, Some(&lb));
        }
        Prop::Checkpoint => gantry(sx, sy, s, fog, Color::from_rgba(200, 40, 40, 255), "CHECKPOINT", None),
        Prop::StartGate => gantry(sx, sy, s, fog, Color::from_rgba(30, 60, 160, 255), "START", None),
        Prop::GoalGate => gantry(sx, sy, s, fog, Color::from_rgba(20, 20, 20, 255), "GOAL", None),
    }
}
