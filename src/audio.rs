//! Tiny procedural synth: three original "radio" tunes, sound effects and an engine hum.
use crate::game::Sfx;
use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, play_sound_once, set_sound_volume, stop_sound};

const RATE: u32 = 22050;

fn wav(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    let data_len = (samples.len() * 2) as u32;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32000.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn midi(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

struct Noise(u32);
impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn add_tone(buf: &mut [f32], start: f32, dur: f32, freq: f32, vol: f32, shape: u8) {
    let s0 = (start * RATE as f32) as usize;
    let n = (dur * RATE as f32) as usize;
    let mut ph = 0.0f32;
    for k in 0..n {
        let i = s0 + k;
        if i >= buf.len() {
            break;
        }
        let t = k as f32 / RATE as f32;
        let env = (t / 0.01).min(1.0) * (1.0 - k as f32 / n as f32).powf(1.5);
        ph = (ph + freq / RATE as f32).fract();
        let w = match shape {
            0 => (ph * std::f32::consts::TAU).sin(),
            1 => if ph < 0.5 { 0.6 } else { -0.6 },
            2 => 4.0 * (ph - 0.5).abs() - 1.0,
            _ => ph * 2.0 - 1.0,
        };
        buf[i] += w * vol * env;
    }
}

/// An original 8-bar tune per station.
pub fn make_music(track: usize) -> Vec<f32> {
    let (bpm, root, prog, lead): (f32, f32, [[f32; 3]; 4], [f32; 16]) = match track {
        0 => (124.0, 60.0, [[0., 4., 7.], [7., 11., 14.], [9., 12., 16.], [5., 9., 12.]], [12., 14., 16., 19., 16., 14., 12., 7., 9., 12., 14., 16., 14., 12., 9., 12.]),
        1 => (138.0, 57.0, [[0., 3., 7.], [-4., 0., 3.], [3., 7., 10.], [-2., 2., 5.]], [12., 15., 19., 15., 17., 15., 12., 10., 12., 15., 19., 22., 19., 17., 15., 12.]),
        _ => (110.0, 62.0, [[0., 3., 7.], [-4., 0., 3.], [-2., 2., 5.], [-5., -2., 2.]], [7., 10., 12., 15., 12., 10., 7., 5., 3., 5., 7., 10., 7., 5., 3., 2.]),
    };
    let beat = 60.0 / bpm;
    let bars = 8;
    let len = bars as f32 * 4.0 * beat;
    let mut buf = vec![0.0f32; (len * RATE as f32) as usize + 1];
    let mut noise = Noise(12345 + track as u32);
    for bar in 0..bars {
        let chord = prog[bar % 4];
        let t0 = bar as f32 * 4.0 * beat;
        // bass: eighth notes
        for e in 0..8 {
            let n = root - 24.0 + chord[0] + if e % 2 == 1 { 12.0 } else { 0.0 };
            add_tone(&mut buf, t0 + e as f32 * beat * 0.5, beat * 0.45, midi(n), 0.16, 2);
        }
        // pad chord
        for c in chord {
            add_tone(&mut buf, t0, beat * 4.0, midi(root + c), 0.035, 3);
        }
        // lead: 16ths arpeggio on odd bars, melody line on even
        for s in 0..16 {
            if bar % 2 == 0 {
                if s % 2 == 0 {
                    let n = root + lead[(s + bar * 2) % 16];
                    add_tone(&mut buf, t0 + s as f32 * beat * 0.25, beat * 0.45, midi(n), 0.07, 1);
                }
            } else {
                let n = root + 12.0 + chord[s % 3];
                add_tone(&mut buf, t0 + s as f32 * beat * 0.25, beat * 0.2, midi(n), 0.045, 1);
            }
        }
        // drums
        for b in 0..4 {
            let tb = t0 + b as f32 * beat;
            // kick
            let s0 = (tb * RATE as f32) as usize;
            for k in 0..(0.15 * RATE as f32) as usize {
                if s0 + k >= buf.len() { break; }
                let t = k as f32 / RATE as f32;
                let f = 120.0 * (-t * 25.0).exp() + 45.0;
                buf[s0 + k] += (t * f * std::f32::consts::TAU).sin() * 0.35 * (1.0 - t / 0.15);
            }
            // hat on off-beat, snare on 2 and 4
            let th = tb + beat * 0.5;
            let s1 = (th * RATE as f32) as usize;
            for k in 0..(0.04 * RATE as f32) as usize {
                if s1 + k >= buf.len() { break; }
                buf[s1 + k] += noise.next() * 0.07 * (1.0 - k as f32 / (0.04 * RATE as f32));
            }
            if b % 2 == 1 {
                for k in 0..(0.12 * RATE as f32) as usize {
                    if s0 + k >= buf.len() { break; }
                    buf[s0 + k] += noise.next() * 0.16 * (1.0 - k as f32 / (0.12 * RATE as f32));
                }
            }
        }
    }
    buf
}

fn sfx(kind: Sfx) -> Vec<f32> {
    let mut noise = Noise(777);
    let mut buf = vec![0.0f32; (RATE as f32 * 1.2) as usize];
    match kind {
        Sfx::Beep => add_tone(&mut buf, 0.0, 0.25, 660.0, 0.4, 1),
        Sfx::Go => add_tone(&mut buf, 0.0, 0.6, 1320.0, 0.4, 1),
        Sfx::Checkpoint | Sfx::Goal => {
            for (k, n) in [72.0, 76.0, 79.0, 84.0].iter().enumerate() {
                add_tone(&mut buf, k as f32 * 0.09, 0.4, midi(*n), 0.3, 1);
            }
        }
        Sfx::Heart => {
            for (k, n) in [79.0, 84.0, 88.0].iter().enumerate() {
                add_tone(&mut buf, k as f32 * 0.07, 0.3, midi(*n), 0.25, 0);
            }
        }
        Sfx::TimeUp => {
            for (k, n) in [67.0, 63.0, 60.0].iter().enumerate() {
                add_tone(&mut buf, k as f32 * 0.25, 0.4, midi(*n), 0.3, 1);
            }
        }
        Sfx::Pop => {
            for k in 0..(0.08 * RATE as f32) as usize {
                buf[k] = noise.next() * 0.5 * (1.0 - k as f32 / (0.08 * RATE as f32));
            }
        }
        Sfx::Crash | Sfx::Bump => {
            let dur = if kind == Sfx::Crash { 1.0 } else { 0.3 };
            let mut lp = 0.0f32;
            for k in 0..(dur * RATE as f32) as usize {
                lp += (noise.next() - lp) * 0.2;
                buf[k] = lp * 1.6 * (1.0 - k as f32 / (dur * RATE as f32)).powi(2);
            }
        }
    }
    buf
}

fn skid_loop() -> Vec<f32> {
    let mut noise = Noise(4242);
    let n = RATE as usize;
    let mut lp = 0.0f32;
    (0..n)
        .map(|k| {
            lp += (noise.next() - lp) * 0.5;
            let t = k as f32 / RATE as f32;
            lp * 0.35 * (0.8 + 0.2 * (t * 60.0 * std::f32::consts::TAU).sin())
        })
        .collect()
}

pub struct Audio {
    pub music: Vec<Sound>,
    sfx: Vec<(Sfx, Sound)>,
    skid: Sound,
    playing: Option<usize>,
    skid_on: bool,
}

pub const STATIONS: [&str; 4] = ["Seaside Breeze", "Turbo Sunset", "Night Glide", "Radio Off"];

impl Audio {
    pub async fn load() -> Option<Audio> {
        let mut music = Vec::new();
        for t in 0..3 {
            music.push(load_sound_from_bytes(&wav(&make_music(t))).await.ok()?);
        }
        let mut s = Vec::new();
        for k in [Sfx::Beep, Sfx::Go, Sfx::Checkpoint, Sfx::Crash, Sfx::Bump, Sfx::Goal, Sfx::TimeUp, Sfx::Heart, Sfx::Pop] {
            s.push((k, load_sound_from_bytes(&wav(&sfx(k))).await.ok()?));
        }
        let skid = load_sound_from_bytes(&wav(&skid_loop())).await.ok()?;
        Some(Audio { music, sfx: s, skid, playing: None, skid_on: false })
    }

    pub fn play(&self, k: Sfx) {
        if let Some((_, s)) = self.sfx.iter().find(|(kk, _)| *kk == k) {
            play_sound_once(s);
        }
    }

    pub fn set_station(&mut self, st: Option<usize>) {
        if st == self.playing {
            return;
        }
        if let Some(p) = self.playing {
            stop_sound(&self.music[p]);
        }
        if let Some(n) = st {
            if n < self.music.len() {
                play_sound(&self.music[n], PlaySoundParams { looped: true, volume: 0.55 });
            }
        }
        self.playing = st.filter(|n| *n < self.music.len());
    }

    pub fn set_skid(&mut self, on: bool) {
        if on && !self.skid_on {
            play_sound(&self.skid, PlaySoundParams { looped: true, volume: 0.5 });
        } else if !on && self.skid_on {
            stop_sound(&self.skid);
        }
        self.skid_on = on;
        if on {
            set_sound_volume(&self.skid, 0.5);
        }
    }
}
