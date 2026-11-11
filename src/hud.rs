//! On-screen information: time, score, lap, speed/gear, route map, challenge task, messages.
use crate::consts::*;
use crate::game::*;
use crate::track::*;
use macroquad::prelude::*;

#[derive(Default, Clone, Copy)]
pub struct HudFlags {
    pub time: bool,
    pub score: bool,
    pub lap: bool,
    pub speed: bool,
    pub map: bool,
}

pub fn txt(s: &str, x: f32, y: f32, size: f32, c: Color) {
    let o = (size / 18.0).max(1.5);
    draw_text(s, x + o, y + o, size, Color::new(0.0, 0.0, 0.0, 0.7 * c.a));
    draw_text(s, x, y, size, c);
}

pub fn txt_c(s: &str, cx: f32, y: f32, size: f32, c: Color) {
    let d = measure_text(s, None, size as u16, 1.0);
    txt(s, cx - d.width * 0.5, y, size, c);
}

pub fn txt_r(s: &str, rx: f32, y: f32, size: f32, c: Color) {
    let d = measure_text(s, None, size as u16, 1.0);
    txt(s, rx - d.width, y, size, c);
}

pub fn fmt_time(t: f32) -> String {
    let cs = (t * 100.0).max(0.0) as u32;
    format!("{}'{:02}\"{:02}", cs / 6000, (cs / 100) % 60, cs % 100)
}

const LABEL: Color = Color::new(1.0, 0.85, 0.2, 1.0);

pub fn heart(x: f32, y: f32, s: f32, c: Color) {
    draw_circle(x - s * 0.5, y, s * 0.55, c);
    draw_circle(x + s * 0.5, y, s * 0.55, c);
    draw_triangle(vec2(x - s * 1.02, y + s * 0.15), vec2(x + s * 1.02, y + s * 0.15), vec2(x, y + s * 1.3), c);
}

pub fn draw_route_map(x0: f32, y0: f32, u: f32, route: &[usize], current: usize, clock: f32, themes: &[Theme]) {
    let dx = 40.0 * u;
    let dy = 24.0 * u;
    let pos = |id: usize| {
        let (r, c) = stage_row(id);
        vec2(x0 + r as f32 * dx, y0 + (c as f32 - r as f32 * 0.5) * dy)
    };
    draw_rectangle(x0 - 16.0 * u, y0 - 70.0 * u, dx * 4.0 + 32.0 * u, 140.0 * u, Color::new(0.0, 0.0, 0.0, 0.35));
    for id in 0..15 {
        let (r, c) = stage_row(id);
        if r + 1 < ROWS {
            for nc in [c, c + 1] {
                let a = pos(id);
                let b = pos(stage_id(r + 1, nc));
                draw_line(a.x, a.y, b.x, b.y, 2.0 * u, Color::new(1.0, 1.0, 1.0, 0.35));
            }
        }
    }
    for w in route.windows(2) {
        let a = pos(w[0]);
        let b = pos(w[1]);
        draw_line(a.x, a.y, b.x, b.y, 4.0 * u, LABEL);
    }
    for id in 0..15 {
        let p = pos(id);
        let visited = route.contains(&id);
        let col = themes[id].ground[0];
        draw_circle(p.x, p.y, 7.0 * u, if visited { LABEL } else { Color::new(0.1, 0.1, 0.1, 0.8) });
        draw_circle(p.x, p.y, 5.0 * u, col);
        if id == current && (clock * 4.0).sin() > -0.3 {
            draw_circle_lines(p.x, p.y, 10.0 * u, 2.5 * u, RED);
        }
    }
}

pub fn draw_hud(r: &Race, themes: &[Theme]) -> HudFlags {
    let w = screen_width();
    let h = screen_height();
    let u = h / 720.0;
    let mut f = HudFlags::default();
    let white = WHITE;

    // --- score (top left) ---
    txt("SCORE", 24.0 * u, 36.0 * u, 26.0 * u, LABEL);
    txt(&format!("{:>9}", r.score as u64), 24.0 * u, 74.0 * u, 42.0 * u, white);
    f.score = true;
    let th = &themes[r.stage.id];
    txt(&format!("STAGE {}  {}", r.stage.row + 1, th.name), 24.0 * u, 104.0 * u, 24.0 * u, white);
    if r.mode == Mode::Challenge {
        for k in 0..ROWS {
            let filled = (k as u32) < r.hearts;
            heart(36.0 * u + k as f32 * 30.0 * u, 124.0 * u, 10.0 * u, if filled { Color::from_rgba(250, 60, 110, 255) } else { Color::new(0.0, 0.0, 0.0, 0.4) });
        }
    }

    // --- time (top centre) ---
    let cx = w * 0.5;
    if r.mode == Mode::TimeAttack {
        txt_c("TIME", cx, 36.0 * u, 28.0 * u, LABEL);
        txt_c(&fmt_time(r.total_time), cx, 88.0 * u, 52.0 * u, white);
    } else {
        txt_c("TIME", cx, 36.0 * u, 28.0 * u, LABEL);
        let t = r.time_left.ceil() as i32;
        let low = r.time_left <= 10.0;
        let col = if low { if (r.clock * 6.0).sin() > 0.0 { RED } else { Color::from_rgba(255, 140, 140, 255) } } else { white };
        txt_c(&format!("{}", t), cx, 104.0 * u, 80.0 * u, col);
    }
    f.time = true;
    if r.mode == Mode::Challenge {
        if let Some(task) = r.task {
            draw_rectangle(cx - 230.0 * u, 116.0 * u, 460.0 * u, 54.0 * u, Color::new(0.0, 0.0, 0.0, 0.35));
            txt_c(&task.text(), cx, 140.0 * u, 24.0 * u, Color::from_rgba(255, 170, 200, 255));
            txt_c(&r.task_progress(), cx, 164.0 * u, 22.0 * u, white);
        }
    }

    // --- lap (top right) ---
    let rx = w - 24.0 * u;
    txt_r("LAP", rx, 36.0 * u, 26.0 * u, LABEL);
    txt_r(&fmt_time(r.stage_time), rx, 72.0 * u, 36.0 * u, white);
    txt_r(&format!("TOTAL {}", fmt_time(r.total_time)), rx, 100.0 * u, 22.0 * u, white);
    f.lap = true;

    // --- speed / tacho (bottom right) ---
    let by = h - 30.0 * u;
    txt_r("km/h", rx, by, 26.0 * u, LABEL);
    txt_r(&format!("{}", r.speed.round() as i32), rx - 64.0 * u, by, 72.0 * u, white);
    let bars = 20;
    let bw = 11.0 * u;
    let bx0 = rx - bars as f32 * (bw + 3.0 * u);
    for k in 0..bars {
        let t = k as f32 / bars as f32;
        let on = t < r.rpm;
        let c = if t > 0.85 { RED } else if t > 0.65 { YELLOW } else { Color::from_rgba(80, 230, 120, 255) };
        let bh = (10.0 + t * 26.0) * u;
        let x = bx0 + k as f32 * (bw + 3.0 * u);
        draw_rectangle(x, by - 80.0 * u - bh, bw, bh, if on { c } else { Color::new(0.0, 0.0, 0.0, 0.35) });
    }
    txt_r(&format!("{} {}", if r.manual { "MT" } else { "AT" }, r.gear + 1), rx, by - 128.0 * u, 30.0 * u, white);
    f.speed = true;

    // --- route map (bottom left) ---
    draw_route_map(48.0 * u, h - 96.0 * u, u, &r.route, r.stage.id, r.clock, themes);
    // stage progress bar
    let prog = (r.z / r.stage.len_m()).clamp(0.0, 1.0);
    draw_rectangle(32.0 * u, h - 20.0 * u, 192.0 * u, 8.0 * u, Color::new(0.0, 0.0, 0.0, 0.4));
    draw_rectangle(32.0 * u, h - 20.0 * u, 192.0 * u * prog, 8.0 * u, LABEL);
    f.map = true;

    // --- messages ---
    if let Phase::Countdown(t) = r.phase {
        let n = t.ceil() as i32;
        txt_c(&format!("{}", n), cx, h * 0.42, 140.0 * u, Color::from_rgba(255, 220, 60, 255));
    } else if let Phase::Racing = r.phase {
        if r.stage.id == 0 && r.stage_time < 1.2 && r.total_time < 1.2 {
            txt_c("GO!", cx, h * 0.42, 140.0 * u, Color::from_rgba(80, 230, 120, 255));
        }
    }
    if let Some((m, t)) = &r.msg {
        let a = t.min(1.0);
        for (k, line) in m.lines().enumerate() {
            txt_c(line, cx, h * 0.3 + k as f32 * 40.0 * u, 38.0 * u, Color::new(1.0, 0.95, 0.4, a));
        }
    }
    if r.stage.has_fork && r.z > (FORK_START as f32 - 60.0) * SEG_LEN && r.z < (FORK_START as f32 + 30.0) * SEG_LEN && (r.clock * 3.0).sin() > 0.0 {
        txt_c("<<  CHOOSE YOUR ROUTE  >>", cx, h * 0.24, 34.0 * u, WHITE);
    }
    if r.drifting {
        txt_c("DRIFT!", cx, h * 0.5, 34.0 * u, Color::from_rgba(120, 220, 255, 255));
    }
    if r.crash_t > 0.0 {
        txt_c("CRASH!", cx, h * 0.5, 40.0 * u, RED);
    }
    match r.phase {
        Phase::TimeUp(_) => {
            txt_c("TIME UP", cx, h * 0.38, 96.0 * u, RED);
            txt_c("GAME OVER", cx, h * 0.48, 48.0 * u, WHITE);
        }
        Phase::Goal(_) => {
            txt_c("GOAL!", cx, h * 0.4, 110.0 * u, Color::from_rgba(255, 220, 60, 255));
        }
        _ => {}
    }
    f
}

pub fn draw_results(r: &Race, themes: &[Theme], best: Option<f32>) {
    let w = screen_width();
    let h = screen_height();
    let u = h / 720.0;
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.05, 0.72));
    let cx = w * 0.5;
    let goal = matches!(r.phase, Phase::Goal(_));
    txt_c(if goal { "COURSE COMPLETE" } else { "GAME OVER" }, cx, 90.0 * u, 60.0 * u, LABEL);
    txt_c(r.mode.name(), cx, 130.0 * u, 28.0 * u, WHITE);
    let mut y = 190.0 * u;
    for (k, id) in r.route.iter().enumerate() {
        let split = r.splits.get(k).map(|t| fmt_time(*t)).unwrap_or_else(|| "--'--\"--".into());
        let mark = if r.mode == Mode::Challenge {
            match r.task_results.get(k) {
                Some(true) => "  <3",
                Some(false) => "  x",
                None => "",
            }
        } else { "" };
        txt(&format!("STAGE {}  {}", k + 1, themes[*id].name), cx - 300.0 * u, y, 30.0 * u, WHITE);
        txt_r(&format!("{}{}", split, mark), cx + 300.0 * u, y, 30.0 * u, WHITE);
        y += 40.0 * u;
    }
    y += 20.0 * u;
    txt(&"TOTAL TIME".to_string(), cx - 300.0 * u, y, 32.0 * u, LABEL);
    txt_r(&fmt_time(r.total_time), cx + 300.0 * u, y, 32.0 * u, WHITE);
    y += 44.0 * u;
    txt("SCORE", cx - 300.0 * u, y, 32.0 * u, LABEL);
    txt_r(&format!("{}", r.score as u64), cx + 300.0 * u, y, 32.0 * u, WHITE);
    y += 44.0 * u;
    if r.mode == Mode::Challenge {
        let rank = ["E", "D", "C", "B", "A", "AAA"][r.hearts.min(5) as usize];
        txt("HEARTS", cx - 300.0 * u, y, 32.0 * u, LABEL);
        txt_r(&format!("{} / 5   RANK {}", r.hearts, rank), cx + 300.0 * u, y, 32.0 * u, WHITE);
        y += 44.0 * u;
    }
    if let Some(b) = best {
        txt("BEST (THIS ROUTE)", cx - 300.0 * u, y, 32.0 * u, LABEL);
        txt_r(&fmt_time(b), cx + 300.0 * u, y, 32.0 * u, WHITE);
    }
    txt_c("PRESS ENTER", cx, h - 50.0 * u, 32.0 * u, WHITE);
}
