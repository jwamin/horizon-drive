//! Race state: driving physics, drift, traffic, checkpoints/forks, modes and scoring.
use crate::consts::*;
use crate::track::*;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Arcade,
    TimeAttack,
    Challenge,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Arcade => "ARCADE RUN",
            Mode::TimeAttack => "TIME ATTACK",
            Mode::Challenge => "PASSENGER CHALLENGE",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Countdown(f32),
    Racing,
    TimeUp(f32),
    Goal(f32),
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub accel: bool,
    pub brake: bool,
    pub steer: f32, // -1 left .. 1 right
    pub gear_up: bool,
    pub gear_down: bool,
    pub toggle_mt: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sfx {
    Beep,
    Go,
    Checkpoint,
    Crash,
    Bump,
    Goal,
    TimeUp,
    Heart,
    Pop,
}

pub struct Traffic {
    pub z: f32,
    pub x: f32,
    pub v: f32,
    pub kind: u8, // 0 sedan 1 hatch 2 van 3 truck 4 bus
    pub color: Color,
    pub ahead: bool,
}

impl Traffic {
    pub fn dims(&self) -> (f32, f32, f32) {
        // width, height, length
        match self.kind {
            0 => (1.85, 1.4, 4.6),
            1 => (1.75, 1.5, 4.0),
            2 => (2.0, 2.2, 5.0),
            3 => (2.4, 3.4, 8.0),
            _ => (2.5, 3.2, 11.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TaskKind {
    Overtake(u32),
    Drift(f32),
    NoHits,
    KeepFast(f32, f32),
    Balloons(u32),
}

impl TaskKind {
    pub fn for_stage(id: usize, row: usize) -> TaskKind {
        let r = row as f32;
        match id % 5 {
            0 => TaskKind::Overtake(10 + 2 * row as u32),
            1 => TaskKind::Drift(250.0 + 80.0 * r),
            2 => TaskKind::Balloons(10 + 2 * row as u32),
            3 => TaskKind::KeepFast(200.0, 14.0 + 3.0 * r),
            _ => TaskKind::NoHits,
        }
    }
    pub fn text(self) -> String {
        match self {
            TaskKind::Overtake(n) => format!("Overtake {} cars!", n),
            TaskKind::Drift(m) => format!("Drift for {:.0} m!", m),
            TaskKind::NoHits => "Don't hit anything!".to_string(),
            TaskKind::KeepFast(k, s) => format!("Stay over {:.0} km/h for {:.0} s!", k, s),
            TaskKind::Balloons(n) => format!("Pop {} balloons!", n),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct StageStats {
    pub overtakes: u32,
    pub drift_m: f32,
    pub hits: u32,
    pub fast_s: f32,
    pub balloons: u32,
}

pub struct Race {
    pub mode: Mode,
    pub car: usize,
    pub stage: Stage,
    pub prev_theme: usize,
    pub route: Vec<usize>,
    pub phase: Phase,
    // car state
    pub z: f32,
    pub x: f32,
    pub speed: f32,
    pub steer: f32,
    pub gear: usize,
    pub manual: bool,
    pub shift_t: f32,
    pub drifting: bool,
    pub drift_dir: f32,
    pub drift_angle: f32,
    pub drift_off_t: f32,
    pub brake_t: f32,
    pub crash_t: f32,
    pub bump_t: f32,
    pub offroad: bool,
    pub braking: bool,
    pub throttle: bool,
    pub rpm: f32,
    // camera / backdrop
    pub cam_x: f32,
    pub bg_offset: f32,
    pub shake: f32,
    // timers and score
    pub time_left: f32,
    pub stage_time: f32,
    pub total_time: f32,
    pub splits: Vec<f32>,
    pub score: f64,
    pub msg: Option<(String, f32)>,
    pub traffic: Vec<Traffic>,
    pub balloons: Vec<(f32, f32, bool)>,
    // challenge
    pub task: Option<TaskKind>,
    pub stats: StageStats,
    pub hearts: u32,
    pub task_results: Vec<bool>,
    // bookkeeping
    pub events: Vec<Sfx>,
    pub checkpoints: u32,
    pub drifts_started: u32,
    pub overtakes_total: u32,
    pub max_speed: f32,
    pub clock: f32,
    pub rng: Rng,
    pub particles: Vec<Particle>,
    pub cp_time_before: f32,
}

pub struct Particle {
    pub sx: f32, // screen-space offset from car, in car-widths
    pub sy: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub dust: bool,
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl Race {
    pub fn new(mode: Mode, car: usize, themes: &[Theme], seed: u64) -> Race {
        let stage = Stage::generate(0, themes);
        let mut r = Race {
            mode,
            car,
            stage,
            prev_theme: 0,
            route: vec![0],
            phase: Phase::Countdown(COUNTDOWN),
            z: 40.0,
            x: LANE_CENTERS[2] - 2.0,
            speed: 0.0,
            steer: 0.0,
            gear: 0,
            manual: false,
            shift_t: 0.0,
            drifting: false,
            drift_dir: 0.0,
            drift_angle: 0.0,
            drift_off_t: 0.0,
            brake_t: 0.0,
            crash_t: 0.0,
            bump_t: 0.0,
            offroad: false,
            braking: false,
            throttle: false,
            rpm: 0.15,
            cam_x: 0.0,
            bg_offset: 0.0,
            shake: 0.0,
            time_left: START_TIME,
            stage_time: 0.0,
            total_time: 0.0,
            splits: Vec::new(),
            score: 0.0,
            msg: None,
            traffic: Vec::new(),
            balloons: Vec::new(),
            task: None,
            stats: StageStats::default(),
            hearts: 0,
            task_results: Vec::new(),
            events: Vec::new(),
            checkpoints: 0,
            drifts_started: 0,
            overtakes_total: 0,
            max_speed: 0.0,
            clock: 0.0,
            rng: Rng::new(seed),
            particles: Vec::new(),
            cp_time_before: 0.0,
        };
        r.cam_x = r.x;
        r.setup_stage(themes);
        r
    }

    pub fn spec(&self) -> &'static CarSpec {
        &CARS[self.car]
    }

    pub fn setup_stage(&mut self, themes: &[Theme]) {
        self.stats = StageStats::default();
        self.traffic.clear();
        self.balloons.clear();
        let th = &themes[self.stage.id];
        if self.mode != Mode::TimeAttack {
            let n = (TRAFFIC_PER_STAGE * th.traffic) as usize;
            let zmax = FORK_START as f32 * SEG_LEN - 60.0;
            for k in 0..n {
                let z = lerp(160.0, zmax, (k as f32 + self.rng.f() * 0.8) / n as f32);
                let lane = self.rng.int(4);
                let kind = match self.rng.int(10) {
                    0..=3 => 0,
                    4..=5 => 1,
                    6..=7 => 2,
                    8 => 3,
                    _ => 4,
                };
                let palette = [
                    Color::from_rgba(200, 40, 40, 255),
                    Color::from_rgba(40, 80, 180, 255),
                    Color::from_rgba(230, 230, 230, 255),
                    Color::from_rgba(40, 40, 45, 255),
                    Color::from_rgba(60, 150, 80, 255),
                    Color::from_rgba(240, 180, 40, 255),
                    Color::from_rgba(150, 150, 160, 255),
                    Color::from_rgba(120, 60, 150, 255),
                ];
                let color = palette[self.rng.int(palette.len())];
                let v = self.rng.range(TRAFFIC_MIN_KMH, TRAFFIC_MAX_KMH) * if kind >= 3 { 0.8 } else { 1.0 };
                self.traffic.push(Traffic { z, x: LANE_CENTERS[lane] + self.rng.range(-0.4, 0.4), v, kind, color, ahead: true });
            }
        }
        if self.mode == Mode::Challenge {
            let t = TaskKind::for_stage(self.stage.id, self.stage.row);
            self.task = Some(t);
            if let TaskKind::Balloons(_) = t {
                let zmax = FORK_START as f32 * SEG_LEN - 40.0;
                for k in 0..25 {
                    let z = lerp(250.0, zmax, k as f32 / 25.0);
                    let lane = self.rng.int(4);
                    self.balloons.push((z, LANE_CENTERS[lane], false));
                }
            }
            self.msg = Some((t.text(), 3.0));
        }
    }

    pub fn ground_y(&self) -> f32 {
        self.stage.ground_y(self.z)
    }

    fn gear_max(&self, g: usize) -> f32 {
        GEAR_MAX_FRAC[g] * self.spec().top
    }

    fn gear_accel(&self, g: usize, v: f32) -> f32 {
        let top = self.spec().top;
        if v >= self.gear_max(g) || v >= top {
            return 0.0; // rev limiter
        }
        let base = self.spec().accel * (1.0 - (v / top).powi(3)).max(ACCEL_FLOOR);
        let lug = if g > 0 && v < self.gear_max(g - 1) * 0.6 { LUG_FACTOR } else { 1.0 };
        base * lug
    }

    /// The autopilot used by the self-test and the attract/screenshot run.
    pub fn autopilot(&self) -> Input {
        let spec = self.spec();
        let split = self.stage.split_at_z(self.z + 30.0);
        let centre = -split; // always take the left branch
        // pick the lane with the most room ahead
        let mut best = (f32::MIN, LANE_CENTERS[1] + centre);
        for lc in LANE_CENTERS {
            let lx = lc + centre;
            let mut room = 400.0f32;
            for t in &self.traffic {
                let dz = t.z - self.z;
                if dz > -6.0 && dz < room && (t.x - lc).abs() < 2.6 {
                    room = dz;
                }
            }
            for b in &self.balloons {
                let dz = b.0 - self.z;
                if !b.2 && dz > 0.0 && dz < 120.0 && (b.1 - lc).abs() < 1.0 {
                    room += 60.0;
                }
            }
            let score = room - (lx - self.x).abs() * 3.0;
            if score > best.0 {
                best = (score, lx);
            }
        }
        let target = best.1;
        let sp = self.speed / spec.top;
        let curve = self.stage.curve_at(self.z + 10.0) / CURVE_HARD;
        let auth = spec.steer * (self.speed / STEER_FULL_KMH).min(1.0).max(0.05);
        let need = curve * sp * sp * CENTRIFUGAL;
        let want = (target - self.x) * 1.6 + need;
        let steer = (want / auth).clamp(-1.0, 1.0);
        let mut brake = false;
        for t in &self.traffic {
            let dz = t.z - self.z;
            if dz > 0.0 && dz < 25.0 && (t.x - self.x).abs() < 2.4 && self.speed > t.v + 20.0 {
                brake = true;
            }
        }
        Input { accel: !brake, brake, steer, ..Default::default() }
    }

    pub fn update(&mut self, dt: f32, inp: Input, themes: &[Theme]) {
        self.clock += dt;
        if let Some((_, t)) = &mut self.msg {
            *t -= dt;
            if *t <= 0.0 {
                self.msg = None;
            }
        }
        match self.phase {
            Phase::Countdown(t) => {
                let nt = t - dt;
                if (nt.ceil() as i32) < (t.ceil() as i32) && nt > 0.0 {
                    self.events.push(Sfx::Beep);
                }
                self.throttle = inp.accel;
                self.rpm = lerp(self.rpm, if inp.accel { 0.85 } else { 0.15 }, (dt * 5.0).min(1.0));
                if nt <= 0.0 {
                    self.phase = Phase::Racing;
                    self.events.push(Sfx::Go);
                } else {
                    self.phase = Phase::Countdown(nt);
                }
                self.update_traffic(dt);
                return;
            }
            Phase::Racing => {
                self.stage_time += dt;
                self.total_time += dt;
                if self.mode != Mode::TimeAttack {
                    self.time_left -= dt;
                    if self.time_left <= 0.0 {
                        self.time_left = 0.0;
                        self.phase = Phase::TimeUp(0.0);
                        self.events.push(Sfx::TimeUp);
                        self.drifting = false;
                    }
                }
                self.physics(dt, inp, themes);
            }
            Phase::TimeUp(t) => {
                self.phase = Phase::TimeUp(t + dt);
                self.physics(dt, Input { brake: self.speed > 0.0, ..Default::default() }, themes);
            }
            Phase::Goal(t) => {
                self.phase = Phase::Goal(t + dt);
                let steer = ((0.0 - self.x) * 0.3).clamp(-0.5, 0.5);
                self.physics(dt, Input { brake: self.speed > 0.0 && t > 1.0, steer, ..Default::default() }, themes);
            }
        }
    }

    fn physics(&mut self, dt: f32, inp: Input, themes: &[Theme]) {
        let spec = self.spec();
        let control = matches!(self.phase, Phase::Racing) && self.crash_t <= 0.0;
        let goal = matches!(self.phase, Phase::Goal(_) | Phase::TimeUp(_));
        let acc_in = control && inp.accel;
        let brk_in = (control || goal) && inp.brake;
        let steer_in = if control || goal { inp.steer.clamp(-1.0, 1.0) } else { 0.0 };
        self.throttle = acc_in;
        self.braking = brk_in;

        // --- drift start: a quick brake tap while steering at speed ---
        if brk_in {
            self.brake_t += dt;
        } else {
            if self.brake_t > 0.0
                && self.brake_t < DRIFT_TAP_MAX
                && steer_in.abs() > 0.5
                && self.speed > DRIFT_MIN_START
                && !self.drifting
                && control
            {
                self.drifting = true;
                self.drift_dir = steer_in.signum();
                self.drift_off_t = 0.0;
                self.drifts_started += 1;
            }
            self.brake_t = 0.0;
        }

        // --- gears ---
        if inp.toggle_mt && control {
            self.manual = !self.manual;
        }
        if self.shift_t > 0.0 {
            self.shift_t -= dt;
        }
        if self.manual {
            if inp.gear_up && self.gear < 5 {
                self.gear += 1;
                self.shift_t = SHIFT_TIME;
            }
            if inp.gear_down && self.gear > 0 {
                self.gear -= 1;
                self.shift_t = SHIFT_TIME;
            }
        } else {
            let bestg = (0..6).find(|g| self.speed < self.gear_max(*g) * AUTO_SHIFT_AT).unwrap_or(5);
            if bestg != self.gear {
                self.gear = bestg;
                self.shift_t = SHIFT_TIME;
            }
        }
        let gmax = self.gear_max(self.gear);
        let target_rpm = (self.speed / gmax).clamp(0.0, 1.05).max(if acc_in { 0.2 } else { 0.12 });
        self.rpm = lerp(self.rpm, target_rpm, (dt * 10.0).min(1.0));

        // --- longitudinal ---
        if self.crash_t > 0.0 {
            self.crash_t -= dt;
            self.speed = (self.speed - 260.0 * dt).max(0.0);
            if self.crash_t <= 0.0 {
                // put the car back on the nearest road
                let cs = self.stage.centers(self.z);
                let mut bestx = 0.0;
                let mut bestd = f32::MAX;
                for c in cs.iter().flatten() {
                    for lc in LANE_CENTERS {
                        let d = (self.x - (c + lc)).abs();
                        if d < bestd {
                            bestd = d;
                            bestx = c + lc;
                        }
                    }
                }
                self.x = bestx;
                self.speed = 0.0;
            }
        } else {
            if acc_in && self.shift_t <= 0.0 {
                self.speed += self.gear_accel(self.gear, self.speed) * dt;
            } else if !acc_in {
                self.speed -= COAST_DECEL * dt;
            }
            if brk_in {
                self.speed -= BRAKE_DECEL * dt;
            }
            if self.drifting {
                self.speed -= DRIFT_DECEL * dt;
            }
            if self.offroad && self.speed > OFFROAD_MAX {
                self.speed -= OFFROAD_DECEL * dt;
            }
            self.speed = self.speed.clamp(0.0, spec.top);
        }
        self.max_speed = self.max_speed.max(self.speed);

        // --- lateral ---
        self.steer = lerp(self.steer, steer_in, (dt * STEER_RESPONSE).min(1.0));
        let sp = self.speed / spec.top;
        let auth = spec.steer * (self.speed / STEER_FULL_KMH).min(1.0) * if self.drifting { DRIFT_STEER_MUL } else { 1.0 };
        if self.crash_t <= 0.0 {
            self.x += self.steer * auth * dt;
        }
        let curve = self.stage.curve_at(self.z) / CURVE_HARD;
        self.x -= curve * sp * sp * CENTRIFUGAL * dt * if self.drifting { DRIFT_CENTRIFUGAL_MUL } else { 1.0 };
        self.bg_offset += curve * sp * dt * 0.06;

        if self.drifting {
            if steer_in.signum() != self.drift_dir || steer_in.abs() < 0.3 {
                self.drift_off_t += dt;
            } else {
                self.drift_off_t = 0.0;
            }
            if self.drift_off_t > DRIFT_RELEASE || self.speed < DRIFT_MIN_KEEP || self.crash_t > 0.0 {
                self.drifting = false;
            }
            let m = self.speed / 3.6 * dt;
            self.stats.drift_m += m;
            self.score += (SCORE_DRIFT_PER_SEC * dt) as f64;
            if self.rng.f() < 0.8 {
                for side in [-0.35f32, 0.35] {
                    self.particles.push(Particle {
                        sx: side,
                        sy: 0.0,
                        vx: side * 0.6 - self.drift_dir * 0.4 + self.rng.range(-0.2, 0.2),
                        vy: self.rng.range(0.2, 0.5),
                        life: 0.7,
                        dust: false,
                    });
                }
            }
        }
        let target_angle = if self.drifting { self.drift_dir * DRIFT_MAX_ANGLE } else if self.crash_t > 0.0 { (self.crash_t * 9.0).sin() * 0.9 } else { self.steer * 0.08 };
        self.drift_angle = lerp(self.drift_angle, target_angle, (dt * 6.0).min(1.0));

        // --- move forward ---
        let old_z = self.z;
        self.z += self.speed / 3.6 * dt;
        self.score += (self.speed * dt * SCORE_PER_KMH_SEC) as f64;
        if self.speed > 200.0 {
            self.stats.fast_s += dt;
        }

        // --- off road ---
        self.offroad = !self.stage.on_road(self.z, self.x);
        if self.offroad && self.speed > 20.0 {
            self.shake = 1.0;
            if self.rng.f() < 0.6 {
                self.particles.push(Particle {
                    sx: self.rng.range(-0.5, 0.5),
                    sy: 0.0,
                    vx: self.rng.range(-0.5, 0.5),
                    vy: self.rng.range(0.3, 0.8),
                    life: 0.6,
                    dust: true,
                });
            }
        } else {
            self.shake *= (1.0 - dt * 6.0).max(0.0);
        }
        let s = self.stage.split_at_z(self.z);
        self.x = self.x.clamp(-s - ROAD_HALF - 40.0, s + ROAD_HALF + 40.0);

        // --- roadside props ---
        if self.crash_t <= 0.0 {
            let i0 = self.stage.seg_index(old_z);
            let i1 = self.stage.seg_index(self.z + CAR_LEN * 0.5);
            'outer: for i in i0..=i1 {
                for p in &self.stage.segs[i].props {
                    let h = p.kind.hit_half();
                    if h > 0.0 && (self.x - p.x).abs() < h + CAR_WIDTH * 0.5 {
                        self.crash();
                        break 'outer;
                    }
                }
            }
        }

        // --- traffic ---
        self.update_traffic(dt);
        if self.bump_t > 0.0 {
            self.bump_t -= dt;
        }
        if self.crash_t <= 0.0 && self.bump_t <= 0.0 {
            let mut hit: Option<usize> = None;
            for (k, t) in self.traffic.iter().enumerate() {
                let (w, _, l) = t.dims();
                let dz = t.z - self.z;
                if dz > -l * 0.5 - CAR_LEN * 0.3 && dz < l * 0.5 + CAR_LEN * 0.5 && (t.x - self.x).abs() < (w + CAR_WIDTH) * 0.5 {
                    hit = Some(k);
                    break;
                }
            }
            if let Some(k) = hit {
                let tv = self.traffic[k].v;
                let tx = self.traffic[k].x;
                self.stats.hits += 1;
                if self.speed - tv > BUMP_CRASH_DIFF {
                    self.crash();
                } else {
                    self.speed = (tv * BUMP_KEEP).min(self.speed);
                    self.drifting = false;
                    self.bump_t = 0.6;
                    self.shake = 1.5;
                    self.x += if self.x < tx { -BUMP_PUSH } else { BUMP_PUSH };
                    self.events.push(Sfx::Bump);
                }
                self.traffic[k].z += 2.0;
                self.traffic[k].v += 15.0;
            }
        }
        // overtakes
        let mut newly = 0;
        for t in &mut self.traffic {
            let ahead = t.z > self.z;
            if t.ahead && !ahead {
                newly += 1;
            }
            t.ahead = ahead;
        }
        if newly > 0 && self.crash_t <= 0.0 {
            self.stats.overtakes += newly;
            self.overtakes_total += newly;
            self.score += (SCORE_OVERTAKE * newly) as f64;
        }
        // balloons
        let (pz, px) = (self.z, self.x);
        let mut popped = 0;
        for b in &mut self.balloons {
            if !b.2 && (b.0 - pz).abs() < 3.0 && (b.1 - px).abs() < 1.8 {
                b.2 = true;
                popped += 1;
            }
        }
        if popped > 0 {
            self.stats.balloons += popped;
            self.events.push(Sfx::Pop);
        }

        // particles
        for p in &mut self.particles {
            p.sx += p.vx * dt;
            p.sy += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        // camera
        self.cam_x = lerp(self.cam_x, self.x, (dt * CAM_FOLLOW).min(1.0));

        // --- checkpoint / goal ---
        if self.z >= self.stage.len_m() && matches!(self.phase, Phase::Racing) {
            self.pass_line(themes);
        }
    }

    fn crash(&mut self) {
        self.crash_t = CRASH_TIME;
        self.drifting = false;
        self.shake = 2.0;
        self.stats.hits += 1;
        self.events.push(Sfx::Crash);
    }

    fn update_traffic(&mut self, dt: f32) {
        let zmax = FORK_START as f32 * SEG_LEN;
        for t in &mut self.traffic {
            t.z += t.v / 3.6 * dt;
        }
        self.traffic.retain(|t| t.z < zmax);
    }

    fn task_passed(&self) -> bool {
        match self.task {
            Some(TaskKind::Overtake(n)) => self.stats.overtakes >= n,
            Some(TaskKind::Drift(m)) => self.stats.drift_m >= m,
            Some(TaskKind::NoHits) => self.stats.hits == 0,
            Some(TaskKind::KeepFast(_, s)) => self.stats.fast_s >= s,
            Some(TaskKind::Balloons(n)) => self.stats.balloons >= n,
            None => false,
        }
    }

    pub fn task_progress(&self) -> String {
        match self.task {
            Some(TaskKind::Overtake(n)) => format!("{}/{}", self.stats.overtakes.min(n), n),
            Some(TaskKind::Drift(m)) => format!("{:.0}/{:.0} m", self.stats.drift_m.min(m), m),
            Some(TaskKind::NoHits) => if self.stats.hits == 0 { "CLEAN".into() } else { "FAILED".into() },
            Some(TaskKind::KeepFast(_, s)) => format!("{:.1}/{:.0} s", self.stats.fast_s.min(s), s),
            Some(TaskKind::Balloons(n)) => format!("{}/{}", self.stats.balloons.min(n), n),
            None => String::new(),
        }
    }

    fn pass_line(&mut self, themes: &[Theme]) {
        self.splits.push(self.stage_time);
        // challenge grading
        if self.mode == Mode::Challenge {
            let ok = self.task_passed();
            self.task_results.push(ok);
            if ok {
                self.hearts += 1;
                self.score += SCORE_TASK_BONUS as f64;
                self.events.push(Sfx::Heart);
            }
        }
        if self.stage.has_fork {
            let left = self.x < 0.0;
            let row = self.stage.row + 1;
            let col = self.stage.col + if left { 0 } else { 1 };
            let shift = if left { -FORK_SPLIT } else { FORK_SPLIT };
            self.x -= shift;
            self.cam_x -= shift;
            self.z -= self.stage.len_m();
            self.prev_theme = self.stage.id;
            let id = stage_id(row, col);
            self.stage = Stage::generate(id, themes);
            self.route.push(id);
            self.checkpoints += 1;
            self.stage_time = 0.0;
            self.score += SCORE_CHECKPOINT as f64;
            let mut text = String::new();
            if self.mode != Mode::TimeAttack {
                let ext = EXTEND[(row - 1).min(3)];
                self.cp_time_before = self.time_left;
                self.time_left += ext;
                text = format!("CHECKPOINT!   +{:.0} SEC", ext);
            } else {
                text.push_str("CHECKPOINT!");
            }
            self.events.push(Sfx::Checkpoint);
            self.setup_stage(themes);
            if self.mode == Mode::Challenge {
                let ok = *self.task_results.last().unwrap_or(&false);
                let t = self.task.unwrap();
                self.msg = Some((format!("{}  {}\n{}", text, if ok { "<3 Nice driving!" } else { "Aww..." }, t.text()), 3.5));
            } else {
                self.msg = Some((text, 2.5));
            }
        } else {
            self.phase = Phase::Goal(0.0);
            if self.mode != Mode::TimeAttack {
                self.score += (self.time_left * SCORE_TIME_BONUS_PER_SEC as f32) as f64;
            }
            self.events.push(Sfx::Goal);
            self.msg = None;
        }
    }

    pub fn finished(&self) -> bool {
        match self.phase {
            Phase::Goal(t) => t > 3.0,
            Phase::TimeUp(t) => t > 3.0,
            _ => false,
        }
    }
}
