use crate::engine::Settings;
use nice_plug::prelude::*;

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum PitchParam {
    #[name = "Root"]
    Root,
    #[name = "Octave"]
    Octave,
    #[name = "Oct + 5th"]
    OctaveFifth,
    #[name = "2 Oct"]
    TwoOctaves,
    #[name = "2 Oct + 3rd"]
    TwoOctavesThird,
    #[name = "2 Oct + 5th"]
    TwoOctavesFifth,
}

impl PitchParam {
    fn harmonic(self) -> u32 {
        self as u32 + 1
    }

    fn from_harmonic(harmonic: u32) -> Self {
        match harmonic {
            0 | 1 => Self::Root,
            2 => Self::Octave,
            3 => Self::OctaveFifth,
            4 => Self::TwoOctaves,
            5 => Self::TwoOctavesThird,
            _ => Self::TwoOctavesFifth,
        }
    }
}

#[derive(Params)]
pub struct ShriekerParams {
    #[id = "bloom"]
    pub bloom: FloatParam,
    #[id = "pitch"]
    pub pitch: EnumParam<PitchParam>,
    #[id = "squabble"]
    pub squabble: FloatParam,
    #[id = "chirp"]
    pub chirp: FloatParam,
    #[id = "mess"]
    pub mess: FloatParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "tone"]
    pub tone: FloatParam,
    #[id = "choke"]
    pub choke: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "stomp"]
    pub stomp: BoolParam,
}

fn percent(name: &str, default: f64) -> FloatParam {
    FloatParam::new(name, default as f32, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_unit("%")
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

impl Default for ShriekerParams {
    fn default() -> Self {
        let s = Settings::default();
        Self {
            bloom: FloatParam::new(
                "Bloom",
                s.bloom as f32,
                FloatRange::Skewed { min: 0.05, max: 4.0, factor: FloatRange::skew_factor(-1.5) },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            pitch: EnumParam::new("Pitch", PitchParam::from_harmonic(s.harmonic)),
            squabble: percent("Squabble", s.squabble),
            chirp: percent("Chirp", s.chirp),
            mess: percent("Mess", s.mess),
            drive: percent("Drive", s.drive),
            tone: FloatParam::new(
                "Tone",
                s.tone as f32,
                FloatRange::Skewed { min: 500.0, max: 12_000.0, factor: FloatRange::skew_factor(-1.5) },
            )
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(1))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            choke: percent("Choke", s.choke),
            level: percent("Level", s.level),
            stomp: BoolParam::new("Stomp", s.stomp),
        }
    }
}

impl ShriekerParams {
    pub fn settings(&self) -> Settings {
        Settings {
            bloom: self.bloom.value() as f64,
            harmonic: self.pitch.value().harmonic(),
            squabble: self.squabble.value() as f64,
            chirp: self.chirp.value() as f64,
            mess: self.mess.value() as f64,
            drive: self.drive.value() as f64,
            tone: self.tone.value() as f64,
            choke: self.choke.value() as f64,
            level: self.level.value() as f64,
            stomp: self.stomp.value(),
        }
    }
}
