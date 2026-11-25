use std::f64::consts::PI;

/// The engine's values, in the units the controls show. The plugin builds
/// them from its params once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Seconds for the feedback to swell in on a held note.
    pub bloom: f64,
    /// Which harmonic of the played note the feedback locks to, from 1 (the
    /// note itself) to 6.
    pub harmonic: u32,
    /// 0 to 1: how much other harmonics fight the main one for control.
    pub squabble: f64,
    /// 0 to 1: how often pitch blips swoop into the note, and how far.
    pub chirp: f64,
    /// 0 to 1: noise, drift, harder clipping and dropouts in the loop.
    pub mess: f64,
    /// 0 to 1: how hard the feedback is distorted on the way out.
    pub drive: f64,
    /// Low-pass corner on the feedback, in Hz.
    pub tone: f64,
    /// 0 to 1: how much picking a note cuts the feedback before it blooms
    /// again.
    pub choke: f64,
    /// 0 to 1: how loud the feedback is next to the dry guitar.
    pub level: f64,
    /// The footswitch. Off fades the feedback out and passes the input.
    pub stomp: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bloom: 0.6,
            harmonic: 2,
            squabble: 0.25,
            chirp: 0.25,
            mess: 0.2,
            drive: 0.4,
            tone: 5000.0,
            choke: 0.6,
            level: 0.6,
            stomp: true,
        }
    }
}

const MIN_HZ: f64 = 70.0;
const MAX_HZ: f64 = 1400.0;
const MAX_VOICE_HZ: f64 = 6000.0;
const TRACK_RATE: f64 = 12_000.0;
const TRACK_WINDOW: usize = 512;
const TRACK_HOP: usize = 64;
const VOICES: usize = 3;
const DETUNE: [f64; VOICES] = [0.0, 0.13, -0.11];

/// The coefficient for a one-pole smoother with a time constant of
/// `seconds`: `x += (target - x) * coef`.
fn coef(sample_rate: f64, seconds: f64) -> f64 {
    1.0 - (-1.0 / (sample_rate * seconds)).exp()
}

/// Zeroes values too small to hear, so anything decaying toward zero never
/// reaches subnormals, which are slow on x86 and which hosts reject in
/// output.
fn flush(x: f64) -> f64 {
    if x.abs() < 1e-30 {
        0.0
    } else {
        x
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// xorshift64*, seeded so renders repeat exactly.
struct Rng(u64);

impl Rng {
    const SEED: u64 = 0x5EED_F00D_CAFE_BABE;

    fn next(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    fn bipolar(&mut self) -> f64 {
        self.next() * 2.0 - 1.0
    }
}

#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl Biquad {
    fn lowpass(sample_rate: f64, hz: f64, q: f64) -> Self {
        let (s, c) = (2.0 * PI * hz / sample_rate).sin_cos();
        let alpha = s / (2.0 * q);
        let a0 = 1.0 + alpha;
        let b0 = (1.0 - c) / 2.0 / a0;
        Self { b0, b1: 2.0 * b0, b2: b0, a1: -2.0 * c / a0, a2: (1.0 - alpha) / a0, z1: 0.0, z2: 0.0 }
    }

    fn tick(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.z1;
        self.z1 = flush(self.b1 * x - self.a1 * y + self.z2);
        self.z2 = flush(self.b2 * x - self.a2 * y);
        y
    }

    fn clear(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

/// Andrew Simper's state-variable filter, which stays stable while its
/// frequency moves every sample.
#[derive(Clone, Copy, Default)]
struct Svf {
    ic1: f64,
    ic2: f64,
}

impl Svf {
    /// Returns (low-pass, band-pass) for `g = tan(pi * hz / sr)`, `k = 1 / Q`.
    fn tick(&mut self, x: f64, g: f64, k: f64) -> (f64, f64) {
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let v3 = x - self.ic2;
        let v1 = a1 * self.ic1 + a2 * v3;
        let v2 = self.ic2 + a2 * self.ic1 + a3 * v3;
        self.ic1 = flush(2.0 * v1 - self.ic1);
        self.ic2 = flush(2.0 * v2 - self.ic2);
        (v2, v1)
    }
}

/// Finds the played note with McLeod's normalized square difference
/// function, on a copy of the input decimated to about 12 kHz.
pub struct PitchTracker {
    factor: usize,
    count: usize,
    aa: [Biquad; 2],
    ring: Vec<f64>,
    pos: usize,
    hop: usize,
    frame: Vec<f64>,
    nsdf: Vec<f64>,
    rate: f64,
    min_lag: usize,
    max_lag: usize,
}

impl PitchTracker {
    pub fn new(sample_rate: f64) -> Self {
        let factor = ((sample_rate / TRACK_RATE).round() as usize).max(1);
        let rate = sample_rate / factor as f64;
        let cutoff = (rate * 0.4).min(sample_rate * 0.45);
        let max_lag = ((rate / MIN_HZ).ceil() as usize).min(TRACK_WINDOW / 2);
        Self {
            factor,
            count: 0,
            aa: [Biquad::lowpass(sample_rate, cutoff, 0.5412), Biquad::lowpass(sample_rate, cutoff, 1.3066)],
            ring: vec![0.0; TRACK_WINDOW],
            pos: 0,
            hop: 0,
            frame: vec![0.0; TRACK_WINDOW],
            nsdf: vec![0.0; max_lag + 2],
            rate,
            min_lag: ((rate / MAX_HZ).floor() as usize).max(2),
            max_lag,
        }
    }

    pub fn reset(&mut self) {
        self.aa.iter_mut().for_each(Biquad::clear);
        self.ring.fill(0.0);
        self.count = 0;
        self.pos = 0;
        self.hop = 0;
    }

    /// Feeds one input sample, and returns the pitch in Hz whenever a new
    /// analysis finds a clear one.
    pub fn push(&mut self, x: f64) -> Option<f64> {
        let y = self.aa[0].tick(x);
        let y = self.aa[1].tick(y);
        self.count += 1;
        if self.count < self.factor {
            return None;
        }
        self.count = 0;
        self.ring[self.pos] = y;
        self.pos = (self.pos + 1) % TRACK_WINDOW;
        self.hop += 1;
        if self.hop < TRACK_HOP {
            return None;
        }
        self.hop = 0;
        self.analyze()
    }

    fn analyze(&mut self) -> Option<f64> {
        for (i, f) in self.frame.iter_mut().enumerate() {
            *f = self.ring[(self.pos + i) % TRACK_WINDOW];
        }
        let f = &self.frame;
        let energy: f64 = f.iter().map(|x| x * x).sum();
        if energy / (TRACK_WINDOW as f64) < 1e-7 {
            return None;
        }

        let mut m = 2.0 * energy;
        for tau in 0..self.nsdf.len() {
            if tau > 0 {
                m -= f[TRACK_WINDOW - tau] * f[TRACK_WINDOW - tau] + f[tau - 1] * f[tau - 1];
            }
            let r: f64 = f[..TRACK_WINDOW - tau].iter().zip(&f[tau..]).map(|(a, b)| a * b).sum();
            self.nsdf[tau] = if m > 1e-12 { 2.0 * r / m } else { 0.0 };
        }

        let mut peaks = [(0usize, 0.0f64); 16];
        let mut count = 0;
        let mut lobe: Option<(usize, f64)> = None;
        let mut was_negative = false;
        for tau in 1..=self.max_lag {
            let n = self.nsdf[tau];
            if n <= 0.0 {
                was_negative = true;
                if let Some(peak) = lobe.take() {
                    if count < peaks.len() {
                        peaks[count] = peak;
                        count += 1;
                    }
                }
            } else if was_negative && tau >= self.min_lag && lobe.is_none_or(|(_, v)| n > v) {
                lobe = Some((tau, n));
            }
        }
        if let Some(peak) = lobe {
            if peak.0 < self.max_lag && count < peaks.len() {
                peaks[count] = peak;
                count += 1;
            }
        }

        let peaks = &peaks[..count];
        let highest = peaks.iter().map(|p| p.1).fold(0.0, f64::max);
        if highest < 0.6 {
            return None;
        }
        let &(tau, _) = peaks.iter().find(|p| p.1 >= 0.9 * highest)?;
        let (a, b, c) = (self.nsdf[tau - 1], self.nsdf[tau], self.nsdf[tau + 1]);
        let denom = a - 2.0 * b + c;
        let offset = if denom.abs() > 1e-12 { (0.5 * (a - c) / denom).clamp(-0.5, 0.5) } else { 0.0 };
        Some(self.rate / (tau as f64 + offset))
    }
}

/// One self-oscillating loop: a band-pass at the target pitch feeding a
/// soft clipper back into itself, like a string and a loud amp.
#[derive(Clone, Copy, Default)]
struct Voice {
    svf: Svf,
    y: f64,
    log_hz: f64,
    weight: f64,
    wobble: f64,
    wobble_target: f64,
}

impl Voice {
    fn tick(&mut self, input: f64, loop_gain: f64, g: f64, k: f64, bias: f64) -> f64 {
        let (_, band) = self.svf.tick(input + loop_gain * self.y, g, k);
        self.y = (k * band + bias).tanh() - bias.tanh();
        self.y
    }
}

/// The plugin's DSP, apart from the host plumbing so tests can run it
/// offline.
pub struct Engine {
    sample_rate: f32,
    sr: f64,
    glide: f64,
    settings: Settings,
    drive: f64,
    tone: f64,
    level: f64,
    /// How far the effect is engaged, gliding between 0 and 1 with the
    /// footswitch.
    wet: f64,
    tracker: PitchTracker,
    recent: [f64; 3],
    pitch: f64,
    env_fast: f64,
    env_slow: f64,
    refractory: usize,
    swell: f64,
    amp: f64,
    loop_gain: f64,
    chirp_semis: f64,
    chirp_decay: f64,
    chirp_env: f64,
    sputter: f64,
    sputter_left: usize,
    dominant: usize,
    switch_left: usize,
    voices: [Voice; VOICES],
    tone_svf: Svf,
    dc_x: f64,
    dc_y: f64,
    rng: Rng,
    c: Coefs,
}

/// Smoother coefficients that depend only on the sample rate.
struct Coefs {
    fast_attack: f64,
    fast_release: f64,
    slow_attack: f64,
    slow_release: f64,
    release: f64,
    amp: f64,
    loop_gain: f64,
    chirp_env: f64,
    sputter: f64,
    wobble: f64,
    dc: f64,
}

impl Coefs {
    fn new(sr: f64) -> Self {
        Self {
            fast_attack: coef(sr, 0.0005),
            fast_release: coef(sr, 0.02),
            slow_attack: coef(sr, 0.03),
            slow_release: coef(sr, 0.3),
            release: coef(sr, 0.06),
            amp: coef(sr, 0.005),
            loop_gain: coef(sr, 0.03),
            chirp_env: coef(sr, 0.08),
            sputter: coef(sr, 0.002),
            wobble: coef(sr, 0.08),
            dc: (-2.0 * PI * 20.0 / sr).exp(),
        }
    }
}

const PICK_FLOOR: f64 = 0.003;
const GATE: f64 = 0.001;
const LOCKED_GATE: f64 = 0.0001;

impl Engine {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate as f64;
        let settings = Settings::default();
        let mut engine = Self {
            sample_rate,
            sr,
            glide: coef(sr, 0.02),
            settings,
            drive: settings.drive,
            tone: settings.tone,
            level: settings.level,
            wet: 1.0,
            tracker: PitchTracker::new(sr),
            recent: [110.0; 3],
            pitch: 110.0,
            env_fast: 0.0,
            env_slow: 0.0,
            refractory: 0,
            swell: 0.0,
            amp: 0.0,
            loop_gain: 0.0,
            chirp_semis: 0.0,
            chirp_decay: 0.0,
            chirp_env: 0.0,
            sputter: 1.0,
            sputter_left: 0,
            dominant: 0,
            switch_left: 0,
            voices: [Voice::default(); VOICES],
            tone_svf: Svf::default(),
            dc_x: 0.0,
            dc_y: 0.0,
            rng: Rng(Rng::SEED),
            c: Coefs::new(sr),
        };
        engine.reset();
        engine
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Jumps to the current settings instead of gliding to them, and
    /// silences the loops.
    pub fn reset(&mut self) {
        let s = self.settings;
        self.drive = s.drive;
        self.tone = s.tone;
        self.level = s.level;
        self.wet = if s.stomp { 1.0 } else { 0.0 };
        self.tracker.reset();
        self.recent = [110.0; 3];
        self.pitch = 110.0;
        self.env_fast = 0.0;
        self.env_slow = 0.0;
        self.refractory = 0;
        self.swell = 0.0;
        self.amp = 0.0;
        self.loop_gain = 0.0;
        self.chirp_semis = 0.0;
        self.chirp_env = 0.0;
        self.sputter = 1.0;
        self.sputter_left = 0;
        self.dominant = 0;
        self.switch_left = 0;
        let harmonics = self.harmonics();
        for (v, h) in self.voices.iter_mut().zip(harmonics) {
            *v = Voice { log_hz: (self.pitch * h).ln(), ..Voice::default() };
        }
        self.voices[0].weight = 1.0;
        self.tone_svf = Svf::default();
        self.dc_x = 0.0;
        self.dc_y = 0.0;
        self.rng = Rng(Rng::SEED);
    }

    pub fn set(&mut self, settings: Settings) {
        self.settings = settings;
    }

    /// The harmonic each voice sits on: the chosen one, the next one up, and
    /// one an octave above (or the 3rd when the chosen one is the root).
    fn harmonics(&self) -> [f64; VOICES] {
        let h = self.settings.harmonic.clamp(1, 6) as f64;
        [h, h + 1.0, if h == 1.0 { 3.0 } else { 2.0 * h }]
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        let s = self.settings;
        let sr = self.sr;
        let wet_target = if s.stomp { 1.0 } else { 0.0 };
        let harmonics = self.harmonics();
        let max_hz = MAX_VOICE_HZ.min(sr * 0.45);
        let note_glide = coef(sr, 0.006 + s.chirp * 0.08);
        let weight_glide = coef(sr, lerp(0.08, 0.006, s.squabble));
        let bloom_step = 1.0 / (s.bloom.max(0.01) * sr);
        let q = lerp(30.0, 4.0, s.mess);
        let k = 1.0 / q;
        let bias = s.mess * 0.6;
        let open_gain = lerp(1.6, 4.0, s.mess);
        let sputter_rate = ((s.mess - 0.5) * 2.0).max(0.0) * 12.0 / sr;
        let refractory = (0.08 * sr) as usize;

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            self.drive += (s.drive - self.drive) * self.glide;
            self.tone += (s.tone - self.tone) * self.glide;
            self.level += (s.level - self.level) * self.glide;
            self.wet += (wet_target - self.wet) * self.glide;
            if !s.stomp && self.wet < 1e-6 {
                self.wet = 0.0;
            }

            let x = (*l as f64 + *r as f64) * 0.5;

            if let Some(hz) = self.tracker.push(x) {
                self.recent = [self.recent[1], self.recent[2], hz];
                let mut sorted = self.recent;
                sorted.sort_by(f64::total_cmp);
                self.pitch = sorted[1];
            }

            let a = x.abs();
            let fast = if a > self.env_fast { self.c.fast_attack } else { self.c.fast_release };
            self.env_fast = flush(self.env_fast + (a - self.env_fast) * fast);
            let slow = if a > self.env_slow { self.c.slow_attack } else { self.c.slow_release };
            self.env_slow = flush(self.env_slow + (a - self.env_slow) * slow);

            let picked = self.refractory == 0 && self.env_fast > PICK_FLOOR && self.env_fast > 2.0 * self.env_slow;
            if picked {
                self.refractory = refractory;
            } else {
                self.refractory = self.refractory.saturating_sub(1);
            }
            let gate = if self.swell > 0.9 { LOCKED_GATE } else { GATE };
            let open = self.env_slow > gate && self.env_fast > 0.1 * self.env_slow;

            if picked {
                self.swell *= 1.0 - s.choke;
            }
            if open {
                self.swell = (self.swell + bloom_step).min(1.0);
            } else {
                self.swell = flush(self.swell - self.swell * self.c.release);
            }

            let spontaneous = open && self.rng.next() < s.chirp * 2.5 / sr;
            let on_pick = picked && self.rng.next() < s.chirp * 1.2;
            if spontaneous || on_pick {
                let size = s.chirp * (5.0 + 19.0 * self.rng.next());
                self.chirp_semis = if self.rng.next() < 0.75 { size } else { -size };
                self.chirp_decay = coef(sr, 0.02 + 0.08 * self.rng.next());
                self.chirp_env = 1.0;
            }
            self.chirp_semis = flush(self.chirp_semis - self.chirp_semis * self.chirp_decay);
            self.chirp_env = flush(self.chirp_env - self.chirp_env * self.c.chirp_env);

            let amp_target = (self.swell * self.swell).max(self.chirp_env * s.chirp * 0.8);
            self.amp = flush(self.amp + (amp_target - self.amp) * self.c.amp);
            let gain_target = if open { open_gain } else { 0.3 };
            self.loop_gain += (gain_target - self.loop_gain) * self.c.loop_gain;

            if self.sputter_left > 0 {
                self.sputter_left -= 1;
            } else if self.rng.next() < sputter_rate {
                self.sputter_left = (lerp(0.005, 0.04, self.rng.next()) * sr) as usize;
            }
            let sputter_target = if self.sputter_left > 0 { 0.0 } else { 1.0 };
            self.sputter += (sputter_target - self.sputter) * self.c.sputter;

            if s.squabble > 0.001 {
                if self.switch_left == 0 {
                    self.dominant = (self.dominant + 1 + (self.rng.next() * 2.0) as usize) % VOICES;
                    let interval = lerp(1.6, 0.05, s.squabble.sqrt()) * (0.3 + 1.4 * self.rng.next());
                    self.switch_left = (interval * sr) as usize;
                } else {
                    self.switch_left -= 1;
                }
            } else {
                self.dominant = 0;
            }

            let excite = if open { x + s.mess * 0.05 * self.rng.bipolar() + 1e-6 * self.rng.bipolar() } else { x };
            let mut mix = 0.0;
            let mut weights = 0.0;
            for (i, v) in self.voices.iter_mut().enumerate() {
                let mut hz = self.pitch * harmonics[i];
                while hz > max_hz {
                    hz *= 0.5;
                }
                v.log_hz += (hz.ln() - v.log_hz) * note_glide;

                if self.rng.next() < 6.0 / sr {
                    v.wobble_target = self.rng.bipolar();
                }
                v.wobble += (v.wobble_target - v.wobble) * self.c.wobble;

                let semis = self.chirp_semis + s.mess * (0.5 * v.wobble + DETUNE[i]);
                let hz = (v.log_hz.exp() * (semis / 12.0).exp2()).clamp(30.0, max_hz);
                let g = (PI * hz / sr).tan();
                let y = v.tick(excite, self.loop_gain, g, k, bias);

                let target = if i == self.dominant { 1.0 } else { s.squabble * 0.35 };
                v.weight += (target - v.weight) * weight_glide;
                let w = v.weight * (1.0 - s.squabble * 0.4 * (0.5 + 0.5 * v.wobble));
                mix += w * y;
                weights += w;
            }
            let mut fb = mix / weights.max(1.0).sqrt() * self.amp * self.sputter;

            let dc = fb - self.dc_x + self.c.dc * self.dc_y;
            self.dc_x = fb;
            self.dc_y = flush(dc);
            fb = dc;

            let drive = 1.0 + self.drive * 19.0;
            fb = (fb * drive).tanh() / drive.tanh();
            let g = (PI * self.tone.min(sr * 0.45) / sr).tan();
            let (low, _) = self.tone_svf.tick(fb, g, std::f64::consts::SQRT_2);
            fb = low * self.level * self.level * self.wet;

            if !fb.is_finite() {
                self.reset();
                fb = 0.0;
            }

            *l = (*l as f64 + fb) as f32;
            *r = (*r as f64 + fb) as f32;
        }
    }
}
