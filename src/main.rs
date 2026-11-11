//! Horizon Drive — an original arcade road racer: beat the checkpoint timer,
//! drift through bends, weave through traffic and pick your route at every fork.
mod audio;
mod consts;
mod game;
mod hud;
mod render;
mod track;

use audio::{Audio, STATIONS};
use consts::*;
use game::*;
use hud::*;
use macroquad::prelude::*;
use std::collections::HashMap;
use track::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Horizon Drive".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: false,
        sample_count: 4,
        platform: miniquad::conf::Platform {
            // Don't wait for the compositor when the window is hidden (screenshot/self-test runs).
            swap_interval: if std::env::args().any(|a| a == "--screenshot" || a == "--self-test") { Some(0) } else { None },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn arg_value(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

const STEP: f32 = 1.0 / 120.0;

fn read_input() -> Input {
    let accel = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::X);
    let brake = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) || is_key_down(KeyCode::Z) || is_key_down(KeyCode::Space);
    let mut steer = 0.0;
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        steer -= 1.0;
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        steer += 1.0;
    }
    Input {
        accel,
        brake,
        steer,
        gear_up: is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::LeftShift),
        gear_down: is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::LeftControl),
        toggle_mt: is_key_pressed(KeyCode::M),
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Title,
    Setup,
    Race,
    Pause,
    Results,
}

fn new_demo(themes: &[Theme]) -> Race {
    let mut d = Race::new(Mode::Arcade, 0, themes, 99);
    d.phase = Phase::Racing;
    d.time_left = 9999.0;
    d.speed = 180.0;
    d.z = 300.0;
    d
}

#[macroquad::main(window_conf)]
async fn main() {
    let themes = themes();
    if let Some(path) = arg_value("--screenshot") {
        run_screenshot(&path, &themes).await;
        return;
    }
    if let Some(path) = arg_value("--self-test") {
        run_self_test(&path, &themes).await;
        return;
    }

    let mut audio = Audio::load().await;
    if std::env::var("HD_DEBUG").is_ok() { eprintln!("audio loaded: {} at {:.2}s", audio.is_some(), get_time()); }
    let mut screen = Screen::Title;
    let mut demo = new_demo(&themes);
    let mut race: Option<Race> = None;
    let mut setup_row = 0usize;
    let mut mode_sel = 0usize;
    let mut car_sel = 0usize;
    let mut radio_sel = 0usize;
    let mut pause_sel = 0usize;
    let mut best: HashMap<usize, f32> = HashMap::new();
    let mut acc = 0.0f32;
    let mut seed = 1u64;
    let modes = [Mode::Arcade, Mode::TimeAttack, Mode::Challenge];

    loop {
        let dt = get_frame_time().min(0.1);
        let enter = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        let esc = is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P);
        let up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        let down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        let left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        let right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);

        clear_background(BLACK);
        match screen {
            Screen::Title | Screen::Setup => {
                // attract-mode drive behind the menus
                acc += dt;
                while acc >= STEP {
                    let inp = demo.autopilot();
                    demo.update(STEP, inp, &themes);
                    demo.events.clear();
                    acc -= STEP;
                }
                if demo.crash_t > 0.0 || matches!(demo.phase, Phase::Goal(_)) || demo.stage.row >= 2 {
                    demo = new_demo(&themes);
                }
                render::draw_world(&demo, &themes);
                let w = screen_width();
                let h = screen_height();
                let u = h / 720.0;
                if screen == Screen::Title {
                    draw_rectangle(0.0, h * 0.18, w, h * 0.3, Color::new(0.0, 0.0, 0.0, 0.35));
                    txt_c("HORIZON DRIVE", w * 0.5, h * 0.33, 110.0 * u, Color::from_rgba(255, 210, 60, 255));
                    txt_c("an arcade road trip", w * 0.5, h * 0.41, 32.0 * u, WHITE);
                    if (get_time() * 2.0).sin() > -0.2 {
                        txt_c("PRESS ENTER", w * 0.5, h * 0.62, 44.0 * u, WHITE);
                    }
                    txt_c("Arrows/WASD: drive   Up: gas   Down: brake   Tap brake while steering: DRIFT", w * 0.5, h * 0.92, 24.0 * u, WHITE);
                    if enter {
                        screen = Screen::Setup;
                    }
                    if is_key_pressed(KeyCode::Escape) {
                        break;
                    }
                } else {
                    draw_rectangle(w * 0.18, h * 0.1, w * 0.64, h * 0.8, Color::new(0.0, 0.0, 0.08, 0.75));
                    txt_c("RACE SETUP", w * 0.5, h * 0.18, 54.0 * u, Color::from_rgba(255, 210, 60, 255));
                    let rows = [
                        ("MODE", modes[mode_sel].name().to_string()),
                        ("CAR", CARS[car_sel].name.to_string()),
                        ("RADIO", STATIONS[radio_sel].to_string()),
                        ("", "START!".to_string()),
                    ];
                    for (k, (l, v)) in rows.iter().enumerate() {
                        let y = h * 0.3 + k as f32 * 52.0 * u;
                        let sel = k == setup_row;
                        let col = if sel { Color::from_rgba(255, 210, 60, 255) } else { WHITE };
                        if !l.is_empty() {
                            txt(l, w * 0.24, y, 34.0 * u, col);
                            txt_c(&format!("{} {} {}", if sel { "<" } else { " " }, v, if sel { ">" } else { " " }), w * 0.56, y, 34.0 * u, col);
                        } else {
                            txt_c(v, w * 0.5, y, 40.0 * u, col);
                        }
                    }
                    let desc = match modes[mode_sel] {
                        Mode::Arcade => "Reach each checkpoint before time runs out. Pick a fork, 5 stages to a goal.",
                        Mode::TimeAttack => "No traffic, no time limit. Set your best time on any route.",
                        Mode::Challenge => "Your passenger sets a task each stage. Earn hearts for a better ending.",
                    };
                    txt_c(desc, w * 0.5, h * 0.56, 22.0 * u, WHITE);
                    // car preview and stats
                    let spec = &CARS[car_sel];
                    render::draw_car_shape(w * 0.36, h * 0.82, 70.0 * u, (get_time() as f32 * 0.8).sin() * 0.3, Color::from_rgba(spec.body.0, spec.body.1, spec.body.2, 255), false, 0.0, get_time() as f32, 0.0);
                    let stats = [("SPEED", (spec.top - 250.0) / 70.0), ("ACCEL", (spec.accel - 20.0) / 18.0), ("HANDLING", (spec.steer - 8.0) / 4.0)];
                    for (k, (n, v)) in stats.iter().enumerate() {
                        let y = h * 0.66 + k as f32 * 34.0 * u;
                        txt(n, w * 0.5, y, 24.0 * u, WHITE);
                        draw_rectangle(w * 0.62, y - 16.0 * u, 160.0 * u, 14.0 * u, Color::new(1.0, 1.0, 1.0, 0.2));
                        draw_rectangle(w * 0.62, y - 16.0 * u, 160.0 * u * v.clamp(0.05, 1.0), 14.0 * u, Color::from_rgba(255, 210, 60, 255));
                    }
                    txt(&format!("{:.0} km/h top", spec.top), w * 0.5, h * 0.66 + 102.0 * u, 22.0 * u, WHITE);
                    if up && setup_row > 0 {
                        setup_row -= 1;
                    }
                    if down && setup_row < 3 {
                        setup_row += 1;
                    }
                    let d = if left { -1i32 } else if right { 1 } else { 0 };
                    if d != 0 {
                        match setup_row {
                            0 => mode_sel = (mode_sel as i32 + d).rem_euclid(3) as usize,
                            1 => car_sel = (car_sel as i32 + d).rem_euclid(3) as usize,
                            2 => {
                                radio_sel = (radio_sel as i32 + d).rem_euclid(4) as usize;
                                if let Some(a) = audio.as_mut() {
                                    a.set_station(if radio_sel < 3 { Some(radio_sel) } else { None });
                                }
                            }
                            _ => {}
                        }
                    }
                    if enter && setup_row == 3 || enter && setup_row < 3 && is_key_down(KeyCode::LeftShift) {
                        seed += 1;
                        race = Some(Race::new(modes[mode_sel], car_sel, &themes, seed * 7919 + get_time() as u64));
                        if let Some(a) = audio.as_mut() {
                            a.set_station(if radio_sel < 3 { Some(radio_sel) } else { None });
                        }
                        screen = Screen::Race;
                        acc = 0.0;
                    } else if enter {
                        setup_row = 3;
                    }
                    if esc {
                        screen = Screen::Title;
                    }
                }
            }
            Screen::Race | Screen::Pause | Screen::Results => {
                let r = race.as_mut().unwrap();
                if screen == Screen::Race {
                    let inp = read_input();
                    acc += dt;
                    let mut first = true;
                    while acc >= STEP {
                        let mut i = inp;
                        if !first {
                            i.gear_up = false;
                            i.gear_down = false;
                            i.toggle_mt = false;
                        }
                        first = false;
                        r.update(STEP, i, &themes);
                        acc -= STEP;
                    }
                    if let Some(a) = audio.as_mut() {
                        for e in r.events.drain(..) {
                            a.play(e);
                        }
                        a.set_skid(r.drifting);
                    } else {
                        r.events.clear();
                    }
                    if r.finished() {
                        if matches!(r.phase, Phase::Goal(_)) && r.mode == Mode::TimeAttack {
                            let key = *r.route.last().unwrap();
                            let e = best.entry(key).or_insert(f32::MAX);
                            if r.total_time < *e {
                                *e = r.total_time;
                            }
                        }
                        screen = Screen::Results;
                        if let Some(a) = audio.as_mut() {
                            a.set_skid(false);
                        }
                    }
                    if esc {
                        screen = Screen::Pause;
                        pause_sel = 0;
                        if let Some(a) = audio.as_mut() {
                            a.set_skid(false);
                        }
                    }
                }
                render::draw_world(r, &themes);
                draw_hud(r, &themes);
                let w = screen_width();
                let h = screen_height();
                let u = h / 720.0;
                if screen == Screen::Pause {
                    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.5));
                    txt_c("PAUSED", w * 0.5, h * 0.38, 80.0 * u, WHITE);
                    for (k, item) in ["RESUME", "QUIT TO TITLE"].iter().enumerate() {
                        let col = if k == pause_sel { Color::from_rgba(255, 210, 60, 255) } else { WHITE };
                        txt_c(item, w * 0.5, h * 0.5 + k as f32 * 50.0 * u, 40.0 * u, col);
                    }
                    if up || down {
                        pause_sel = 1 - pause_sel;
                    }
                    if esc {
                        screen = Screen::Race;
                    } else if enter {
                        if pause_sel == 0 {
                            screen = Screen::Race;
                        } else {
                            screen = Screen::Title;
                            demo = new_demo(&themes);
                        }
                    }
                    acc = 0.0;
                } else if screen == Screen::Results {
                    let b = if r.mode == Mode::TimeAttack { best.get(r.route.last().unwrap()).copied() } else { None };
                    draw_results(r, &themes, b);
                    if enter || esc {
                        screen = Screen::Setup;
                        demo = new_demo(&themes);
                    }
                }
            }
        }
        next_frame().await;
    }
}

// ---------------------------------------------------------------------------
// --screenshot <file.png>
// ---------------------------------------------------------------------------
async fn run_screenshot(path: &str, themes: &[Theme]) {
    // Optional developer overrides (for checking other stages): HD_STAGE, HD_Z, HD_MODE, HD_CAR.
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
    let mode = [Mode::Arcade, Mode::TimeAttack, Mode::Challenge][env("HD_MODE").unwrap_or(0.0) as usize % 3];
    let mut r = Race::new(mode, env("HD_CAR").unwrap_or(0.0) as usize % 3, themes, 7);
    if let Some(id) = env("HD_STAGE") {
        let id = id as usize % 15;
        r.stage = Stage::generate(id, themes);
        r.prev_theme = id;
        r.route = vec![id];
        r.setup_stage(themes);
    }
    r.phase = Phase::Racing;
    r.z = env("HD_Z").unwrap_or(700.0);
    r.speed = 210.0;
    for frame in 0..120 {
        for _ in 0..2 {
            let inp = r.autopilot();
            r.update(STEP, inp, themes);
        }
        r.events.clear();
        clear_background(BLACK);
        render::draw_world(&r, themes);
        draw_hud(&r, themes);
        if frame == 119 {
            let img = get_screen_data();
            let _ = flip_vertical;
            if let Some(dir) = std::path::Path::new(path).parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            img.export_png(path);
        }
        next_frame().await;
    }
}

fn flip_vertical(img: Image) -> Image {
    let w = img.width as usize;
    let h = img.height as usize;
    let mut out = img.clone();
    for y in 0..h {
        let src = &img.bytes[y * w * 4..(y + 1) * w * 4];
        out.bytes[(h - 1 - y) * w * 4..(h - y) * w * 4].copy_from_slice(src);
    }
    out
}

// ---------------------------------------------------------------------------
// --self-test <file.json>
// ---------------------------------------------------------------------------
struct StepResult {
    step: &'static str,
    passed: bool,
    detail: String,
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ")
}

async fn run_self_test(path: &str, themes: &[Theme]) {
    let mut results: Vec<StepResult> = Vec::new();
    let dt = 1.0 / 60.0;
    let mut r = Race::new(Mode::Arcade, 0, themes, 2024);

    // Helper: run n simulation steps with an input function, drawing a frame now and then.
    async fn sim<F: FnMut(&Race) -> Input>(r: &mut Race, themes: &[Theme], secs: f32, dt: f32, mut f: F, stop: &dyn Fn(&Race) -> bool) {
        let steps = (secs / dt) as usize;
        for k in 0..steps {
            let inp = f(r);
            r.update(dt, inp, themes);
            r.events.clear();
            if stop(r) {
                break;
            }
            if k % 400 == 399 {
                clear_background(BLACK);
                render::draw_world(r, themes);
                draw_hud(r, themes);
                next_frame().await;
            }
        }
    }
    let never = |_: &Race| false;

    // 1. countdown -> racing (holding the gas like a player would)
    let t0 = r.time_left;
    sim(&mut r, themes, COUNTDOWN + 0.1, dt, |_| Input { accel: true, ..Default::default() }, &never).await;
    results.push(StepResult {
        step: "start",
        passed: r.phase == Phase::Racing && (r.time_left - t0).abs() < 0.2,
        detail: format!("countdown finished, phase {:?}, timer {:.1}s", r.phase, r.time_left),
    });

    // 2. move: accelerate for 6 s
    let z0 = r.z;
    sim(&mut r, themes, 6.0, dt, |r| Input { accel: true, steer: r.autopilot().steer, ..Default::default() }, &never).await;
    let moved = r.z - z0;
    results.push(StepResult {
        step: "move",
        passed: moved > 100.0 && r.speed > 100.0,
        detail: format!("drove {:.0} m in 6 s, now {:.0} km/h in gear {}", moved, r.speed, r.gear + 1),
    });

    // 3. steer
    let x0 = r.x;
    let dir = if r.x > 0.0 { -1.0 } else { 1.0 };
    sim(&mut r, themes, 0.5, dt, |_| Input { accel: true, steer: dir, ..Default::default() }, &never).await;
    let dx = (r.x - x0) * dir;
    results.push(StepResult { step: "steer", passed: dx > 1.0, detail: format!("steered {:.1} m sideways in 0.5 s", dx) });

    // 4. act: drift (tap brake while steering at speed)
    sim(&mut r, themes, 8.0, dt, |r| Input { accel: true, steer: r.autopilot().steer, ..Default::default() }, &|r: &Race| r.speed > 170.0).await;
    // line up in the lane on the opposite side so the drift has room
    let d0 = r.drifts_started;
    let ddir = if r.x > 0.0 { -1.0 } else { 1.0 };
    let speed_before = r.speed;
    sim(&mut r, themes, 0.1, dt, |_| Input { accel: true, brake: true, steer: ddir, ..Default::default() }, &never).await;
    let mut was_drifting = false;
    let mut drift_m = 0.0;
    let dm0 = r.stats.drift_m;
    for _ in 0..30 {
        r.update(dt, Input { accel: true, steer: ddir, ..Default::default() }, themes);
        r.events.clear();
        was_drifting |= r.drifting;
        drift_m = r.stats.drift_m - dm0;
    }
    results.push(StepResult {
        step: "act",
        passed: r.drifts_started > d0 && was_drifting && drift_m > 5.0,
        detail: format!("brake tap at {:.0} km/h started a drift, slid {:.0} m, speed now {:.0} km/h", speed_before, drift_m, r.speed),
    });
    // straighten out
    sim(&mut r, themes, 1.0, dt, |r| r.autopilot(), &never).await;

    // 5. traffic collision slows the car
    let sp = r.speed;
    let car_x = r.x;
    r.traffic.push(Traffic { z: r.z + 4.0, x: car_x, v: 80.0, kind: 0, color: RED, ahead: true });
    let hits0 = r.stats.hits;
    sim(&mut r, themes, 0.3, dt, |_| Input { accel: true, ..Default::default() }, &never).await;
    results.push(StepResult {
        step: "collision",
        passed: r.stats.hits > hits0 && r.speed < sp - 30.0,
        detail: format!("rear-ended traffic at {:.0} km/h, speed dropped to {:.0} km/h", sp, r.speed),
    });
    sim(&mut r, themes, 3.0, dt, |r| r.autopilot(), &|r: &Race| r.crash_t <= 0.0 && r.bump_t <= 0.0).await;

    // 6. timer counts down while racing
    let tl = r.time_left;
    sim(&mut r, themes, 2.0, dt, |r| r.autopilot(), &never).await;
    results.push(StepResult {
        step: "timer",
        passed: r.time_left < tl - 1.9,
        detail: format!("time left went from {:.1} to {:.1}", tl, r.time_left),
    });

    // 7. drive to the first checkpoint and take a fork
    let ov0 = r.overtakes_total;
    sim(&mut r, themes, 140.0, dt, |r| r.autopilot(), &|r: &Race| r.checkpoints >= 1 || matches!(r.phase, Phase::TimeUp(_))).await;
    let cp_ok = r.checkpoints >= 1;
    results.push(StepResult {
        step: "checkpoint",
        passed: cp_ok && r.time_left > r.cp_time_before + EXTEND[0] - 1.0,
        detail: format!(
            "reached checkpoint {} after {} with {:.1}s left, extended to {:.1}s (+{:.0}s); overtook {} cars",
            r.checkpoints, fmt_time(r.splits[0]), r.cp_time_before, r.time_left, EXTEND[0], r.overtakes_total - ov0
        ),
    });
    let route_ok = r.route.len() == 2 && r.route[1] == stage_id(1, 0);
    results.push(StepResult {
        step: "fork",
        passed: route_ok,
        detail: format!("took the left branch into stage 2: {}", themes[*r.route.last().unwrap()].name),
    });

    // 8. HUD drawn
    clear_background(BLACK);
    render::draw_world(&r, themes);
    let flags = draw_hud(&r, themes);
    next_frame().await;
    results.push(StepResult {
        step: "hud",
        passed: flags.time && flags.score && flags.speed && flags.lap && flags.map,
        detail: format!(
            "drawn: time {} ({:.0}s), score {} ({}), lap {}, speed/gear {} ({:.0} km/h), route map {}",
            flags.time, r.time_left, flags.score, r.score as u64, flags.lap, flags.speed, r.speed, flags.map
        ),
    });

    // 9. modes
    let ta = Race::new(Mode::TimeAttack, 1, themes, 5);
    let ch = Race::new(Mode::Challenge, 2, themes, 5);
    results.push(StepResult {
        step: "modes",
        passed: ta.traffic.is_empty() && ch.task.is_some() && !r.traffic.is_empty(),
        detail: format!("time attack has {} traffic cars; challenge task: {}", ta.traffic.len(), ch.task.map(|t| t.text()).unwrap_or_default()),
    });

    let mut out = String::from("{\"version\": 1, \"steps\": [\n");
    for (k, s) in results.iter().enumerate() {
        out.push_str(&format!(
            "  {{\"step\": \"{}\", \"passed\": {}, \"detail\": \"{}\"}}{}\n",
            s.step,
            s.passed,
            json_escape(&s.detail),
            if k + 1 < results.len() { "," } else { "" }
        ));
    }
    out.push_str("]}\n");
    if let Some(dir) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, out);
}
