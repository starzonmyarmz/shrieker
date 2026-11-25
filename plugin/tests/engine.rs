mod support;

use shrieker::engine::{Engine, PitchTracker, Settings};

const SR: f32 = 48_000.0;

/// Squabble, chirp and mess off, so the feedback is one steady loop.
const SIMPLE: Settings = Settings {
    bloom: 0.3,
    harmonic: 2,
    squabble: 0.0,
    chirp: 0.0,
    mess: 0.0,
    drive: 0.4,
    tone: 5000.0,
    choke: 0.0,
    level: 0.6,
    stomp: true,
};

/// Runs mono input through the engine in `block`-sized chunks and returns
/// the left output. The engine starts at `settings`.
fn render_blocks(input: &[f32], settings: Settings, block: usize) -> Vec<f32> {
    let mut engine = Engine::new(SR);
    engine.set(settings);
    engine.reset();
    let mut left = input.to_vec();
    let mut right = input.to_vec();
    for (l, r) in left.chunks_mut(block).zip(right.chunks_mut(block)) {
        engine.process(l, r);
    }
    left
}

fn render(input: &[f32], settings: Settings) -> Vec<f32> {
    render_blocks(input, settings, 512)
}

fn wet(input: &[f32], output: &[f32]) -> Vec<f64> {
    output.iter().zip(input).map(|(o, i)| (o - i) as f64).collect()
}

fn rms(x: &[f64]) -> f64 {
    (x.iter().map(|v| v * v).sum::<f64>() / x.len() as f64).sqrt()
}

/// The power of `x` at `hz`.
fn goertzel(x: &[f64], hz: f64) -> f64 {
    let w = 2.0 * std::f64::consts::PI * hz / SR as f64;
    let c = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0, 0.0);
    for &v in x {
        let s = v + c * s1 - s2;
        s2 = s1;
        s1 = s;
    }
    s1 * s1 + s2 * s2 - c * s1 * s2
}

#[test]
fn default_feedback_matches_golden() {
    let input = support::tone(110.0, 1.5, SR);
    support::check_golden("default-feedback", &render(&input, Settings::default()), SR as u32);
}

#[test]
fn tracker_finds_the_played_note() {
    for hz in [82.4, 110.0, 196.0, 659.3] {
        let mut tracker = PitchTracker::new(SR as f64);
        let found = support::tone(hz, 0.3, SR).iter().filter_map(|&x| tracker.push(x as f64)).last().unwrap();
        assert!((found / hz - 1.0).abs() < 0.01, "{hz} Hz tracked as {found} Hz");
    }
}

#[test]
fn silence_stays_silent() {
    let input = vec![0.0; SR as usize];
    assert!(render(&input, Settings { mess: 1.0, chirp: 1.0, ..Settings::default() }).iter().all(|&x| x == 0.0));
}

#[test]
fn held_note_blooms_on_the_chosen_harmonic() {
    let input = support::tone(110.0, 2.0, SR);
    let tail = &wet(&input, &render(&input, SIMPLE))[input.len() * 3 / 4..];
    assert!(rms(tail) > 0.05, "feedback too quiet: {}", rms(tail));
    let octave = goertzel(tail, 220.0);
    assert!(octave > 10.0 * goertzel(tail, 110.0), "root is louder than the octave");
    assert!(octave > 10.0 * goertzel(tail, 330.0), "the 3rd harmonic is louder than the octave");
}

#[test]
fn feedback_takes_bloom_to_swell_in() {
    let input = support::tone(110.0, 2.0, SR);
    let out = wet(&input, &render(&input, Settings { bloom: 1.5, ..SIMPLE }));
    let early = rms(&out[..(0.3 * SR) as usize]);
    let late = rms(&out[(1.6 * SR) as usize..]);
    assert!(late > 4.0 * early, "early {early}, late {late}");
}

#[test]
fn muting_the_strings_stops_the_feedback() {
    let mut input = support::tone(110.0, 1.5, SR);
    input.resize(input.len() + SR as usize / 2, 0.0);
    let out = render(&input, Settings::default());
    let tail: Vec<f64> = out[out.len() - (0.2 * SR) as usize..].iter().map(|&x| x as f64).collect();
    assert!(rms(&tail) < 1e-4, "still ringing: {}", rms(&tail));
}

#[test]
fn feedback_peak_follows_the_feedback() {
    let mut engine = Engine::new(SR);
    let mut silence = vec![0.0; 512];
    engine.process(&mut silence.clone(), &mut silence);
    assert_eq!(engine.feedback_peak(), 0.0);

    let mut input = support::tone(110.0, 2.0, SR);
    input.truncate(input.len() / 512 * 512);
    let mut left = input.clone();
    let mut right = input.clone();
    let mut peak = 0.0;
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.process(l, r);
        peak = engine.feedback_peak();
    }
    let tail = wet(&input[input.len() - 512..], &left[left.len() - 512..]);
    let loudest = tail.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    assert!(peak > 0.05);
    assert!((peak - loudest).abs() < 1e-6, "peak {peak}, loudest {loudest}");
}

#[test]
fn block_size_does_not_change_the_sound() {
    let input = support::tone(110.0, 0.5, SR);
    assert_eq!(render_blocks(&input, Settings::default(), 64), render_blocks(&input, Settings::default(), 512));
}

#[test]
fn stomp_off_fades_to_the_dry_signal() {
    let input = support::tone(110.0, 1.0, SR);
    let mut engine = Engine::new(SR);
    let mut left = input.clone();
    let mut right = input.clone();
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.process(l, r);
    }
    engine.set(Settings { stomp: false, ..Settings::default() });
    let mut left = input.clone();
    let mut right = input.clone();
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.process(l, r);
    }
    let tail = input.len() / 2;
    assert_eq!(left[tail..], input[tail..]);
}
