use crate::bridge;
use crate::engine::Settings;
use crate::params::ShriekerParams;
use serde_json::Value;
use std::sync::Arc;

/// What a preset doesn't set. Loading one leaves Stomp alone.
const BASE: Settings = Settings {
    bloom: 0.6,
    harmonic: 2,
    squabble: 0.0,
    chirp: 0.0,
    mess: 0.0,
    drive: 0.4,
    tone: 5000.0,
    choke: 0.5,
    level: 0.6,
    stomp: true,
};

/// The factory presets, in the order the page shows them.
pub const PRESETS: &[(&str, Settings)] = &[
    ("Simple Howl", Settings { bloom: 0.8, harmonic: 2, drive: 0.3, tone: 4000.0, choke: 0.3, level: 0.55, ..BASE }),
    (
        "Wall of Amps",
        Settings {
            bloom: 2.0,
            harmonic: 3,
            squabble: 0.15,
            chirp: 0.05,
            mess: 0.15,
            drive: 0.6,
            tone: 3500.0,
            choke: 0.2,
            level: 0.65,
            ..BASE
        },
    ),
    ("Squabble", Settings { bloom: 0.3, squabble: 0.85, chirp: 0.2, mess: 0.3, drive: 0.5, tone: 5500.0, ..BASE }),
    (
        "Chirper",
        Settings {
            bloom: 1.2,
            harmonic: 4,
            squabble: 0.2,
            chirp: 0.9,
            mess: 0.15,
            tone: 7000.0,
            choke: 0.7,
            level: 0.55,
            ..BASE
        },
    ),
    (
        "Choke Stabs",
        Settings {
            bloom: 0.25,
            harmonic: 4,
            squabble: 0.3,
            chirp: 0.4,
            mess: 0.35,
            drive: 0.6,
            tone: 6000.0,
            choke: 1.0,
            ..BASE
        },
    ),
    (
        "Whistle",
        Settings { harmonic: 6, squabble: 0.1, chirp: 0.15, mess: 0.05, drive: 0.2, tone: 9000.0, level: 0.45, ..BASE },
    ),
    (
        "Mathcore Mess",
        Settings {
            bloom: 0.15,
            harmonic: 3,
            squabble: 0.7,
            chirp: 0.6,
            mess: 0.85,
            drive: 0.8,
            tone: 6000.0,
            choke: 0.4,
            level: 0.65,
            ..BASE
        },
    ),
    (
        "Broken Cable",
        Settings { bloom: 0.1, squabble: 0.4, chirp: 0.3, mess: 1.0, drive: 0.9, tone: 3000.0, choke: 0.2, ..BASE },
    ),
];

/// The presets as the page gets them, with normalized values.
pub fn json() -> Value {
    PRESETS
        .iter()
        .map(|&(name, settings)| bridge::preset(name, Arc::new(ShriekerParams::new(settings)), &["stomp"]))
        .collect()
}
