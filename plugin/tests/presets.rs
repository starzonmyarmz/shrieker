use shrieker::params::ShriekerParams;
use shrieker::presets::{json, PRESETS};

#[test]
fn presets_fit_the_controls() {
    for &(name, preset) in PRESETS {
        let s = ShriekerParams::new(preset).settings();
        let close = |a: f64, b: f64| (a - b).abs() <= 1e-5 * b.abs().max(1.0);
        assert_eq!(s.harmonic, preset.harmonic, "{name}: pitch");
        for (control, got, want) in [
            ("bloom", s.bloom, preset.bloom),
            ("squabble", s.squabble, preset.squabble),
            ("chirp", s.chirp, preset.chirp),
            ("mess", s.mess, preset.mess),
            ("drive", s.drive, preset.drive),
            ("tone", s.tone, preset.tone),
            ("choke", s.choke, preset.choke),
            ("level", s.level, preset.level),
        ] {
            assert!(close(got, want), "{name}: {control} is {got}, not {want}");
        }
    }
}

#[test]
fn presets_leave_stomp_alone() {
    for preset in json().as_array().unwrap() {
        let values = preset["values"].as_object().unwrap();
        assert!(!values.contains_key("stomp"));
        assert_eq!(values.len(), 9);
    }
}
