//! Every number that shapes how the game plays. Mirrors MECHANICS.md / mechanics.json.
//! All values are estimated (clean room: nothing from the original was read).

// ---- World / road ----
pub const SEG_LEN: f32 = 5.0; // metres per road segment
pub const ROAD_HALF: f32 = 8.0; // half road width (4 lanes of 4 m)
pub const LANE_CENTERS: [f32; 4] = [-6.0, -2.0, 2.0, 6.0];
pub const RUMBLE_W: f32 = 1.2;
pub const STAGE_SEGS: usize = 880; // 4.4 km per stage
pub const FORK_SEGS: usize = 120; // last 600 m: the road splits in two
pub const TAIL_SEGS: usize = 280; // drawn beyond the checkpoint (look-ahead only)
pub const FORK_SPLIT: f32 = 28.0; // each branch centre is this far from the old centre at the checkpoint
pub const CURVE_HARD: f32 = 0.024; // curve value of the tightest bend
pub const ROWS: usize = 5; // stages per run (15 stages in a 1-2-3-4-5 pyramid)

// ---- Camera ----
pub const DRAW_SEGS: usize = 260; // 1.3 km draw distance
pub const CAM_HEIGHT: f32 = 2.8;
pub const CAM_BACK: f32 = 6.5;
pub const FOV_V_DEG: f32 = 50.0;
pub const HORIZON_FRAC: f32 = 0.42;
pub const CAM_FOLLOW: f32 = 6.0; // how fast the camera follows the car sideways (1/s)

// ---- Timer / rules ----
pub const START_TIME: f32 = 75.0;
pub const EXTEND: [f32; 4] = [70.0, 68.0, 66.0, 64.0]; // time added at checkpoints 1..4
pub const COUNTDOWN: f32 = 3.0;

// ---- Driving ----
pub const COAST_DECEL: f32 = 9.0; // km/h per s, off the throttle
pub const BRAKE_DECEL: f32 = 95.0;
pub const OFFROAD_MAX: f32 = 110.0; // grass/sand slows you to this
pub const OFFROAD_DECEL: f32 = 75.0;
pub const CENTRIFUGAL: f32 = 13.0; // m/s outward push in the tightest bend at top speed
pub const STEER_FULL_KMH: f32 = 80.0; // steering reaches full strength at this speed
pub const STEER_RESPONSE: f32 = 8.0; // how fast the wheel follows the keys (1/s)
pub const SHIFT_TIME: f32 = 0.15;
pub const GEAR_MAX_FRAC: [f32; 6] = [0.24, 0.41, 0.59, 0.74, 0.88, 1.0];
pub const ACCEL_FLOOR: f32 = 0.12; // acceleration never drops below 12% until top speed
pub const LUG_FACTOR: f32 = 0.55; // manual: too high a gear for the speed pulls this much
pub const AUTO_SHIFT_AT: f32 = 0.96; // automatic shifts up at 96% of a gear's top

// ---- Drift ----
pub const DRIFT_TAP_MAX: f32 = 0.30; // brake tap shorter than this while steering starts a drift
pub const DRIFT_MIN_START: f32 = 110.0;
pub const DRIFT_MIN_KEEP: f32 = 85.0;
pub const DRIFT_DECEL: f32 = 6.0;
pub const DRIFT_STEER_MUL: f32 = 1.35;
pub const DRIFT_CENTRIFUGAL_MUL: f32 = 0.55;
pub const DRIFT_RELEASE: f32 = 0.25; // drift ends after this long without steering into it
pub const DRIFT_MAX_ANGLE: f32 = 0.45; // radians, how sideways the car looks

// ---- Collisions ----
pub const CRASH_TIME: f32 = 2.5;
pub const BUMP_KEEP: f32 = 0.75; // speed after rear-ending traffic = their speed x this
pub const BUMP_CRASH_DIFF: f32 = 140.0; // hitting traffic this much faster than it = spin-out
pub const BUMP_PUSH: f32 = 1.4;
pub const CAR_WIDTH: f32 = 1.9;
pub const CAR_LEN: f32 = 4.4;

// ---- Traffic ----
pub const TRAFFIC_PER_STAGE: f32 = 34.0;
pub const TRAFFIC_MIN_KMH: f32 = 70.0;
pub const TRAFFIC_MAX_KMH: f32 = 140.0;

// ---- Score ----
pub const SCORE_PER_KMH_SEC: f32 = 3.0;
pub const SCORE_OVERTAKE: u32 = 500;
pub const SCORE_DRIFT_PER_SEC: f32 = 300.0;
pub const SCORE_CHECKPOINT: u32 = 20000;
pub const SCORE_TIME_BONUS_PER_SEC: u32 = 10000;
pub const SCORE_TASK_BONUS: u32 = 50000;

pub struct CarSpec {
    pub name: &'static str,
    pub top: f32,   // km/h
    pub accel: f32, // km/h per s, base
    pub steer: f32, // m/s sideways at full lock
    pub body: (u8, u8, u8),
}

pub const CARS: [CarSpec; 3] = [
    CarSpec { name: "Rossa Barchetta", top: 290.0, accel: 30.0, steer: 10.0, body: (205, 18, 24) },
    CarSpec { name: "Vento Speedster", top: 312.0, accel: 26.0, steer: 9.0, body: (40, 95, 205) },
    CarSpec { name: "Piccola Roadster", top: 272.0, accel: 35.0, steer: 11.5, body: (250, 190, 30) },
];
