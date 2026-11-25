pub mod bridge;
mod editor;
pub mod engine;
pub mod params;

use engine::Engine;
use nice_plug::prelude::*;
use params::ShriekerParams;
use std::sync::Arc;

static PAGE: editor::Page = editor::Page {
    title: "Shrieker",
    size: (600, 340),
    html: include_str!("../../ui/index.html"),
    version: match option_env!("PLUGIN_VERSION") {
        Some(version) => version,
        None => env!("CARGO_PKG_VERSION"),
    },
    background: (27, 26, 25, 255),
    dev_url_var: "SHRIEKER_UI_URL",
};

pub struct Shrieker {
    params: Arc<ShriekerParams>,
    engine: Engine,
}

impl Default for Shrieker {
    fn default() -> Self {
        Self { params: Arc::new(ShriekerParams::default()), engine: Engine::new(48_000.0) }
    }
}

impl Plugin for Shrieker {
    const NAME: &'static str = "Shrieker";
    const VENDOR: &'static str = "Daniel Marino";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type Editor = editor::WebEditor;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        Some(editor::WebEditor::new(&PAGE, self.params.clone()))
    }

    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        if self.engine.sample_rate() != buffer_config.sample_rate {
            self.engine = Engine::new(buffer_config.sample_rate);
        }
        self.reset();
        true
    }

    fn reset(&mut self) {
        self.engine.set(self.params.settings());
        self.engine.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set(self.params.settings());
        if let [left, right] = buffer.as_slice() {
            self.engine.process(left, right);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for Shrieker {
    const CLAP_ID: &'static str = "com.iamdanielmarino.shrieker";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Guitar feedback that blooms, chirps, squabbles and jumps harmonics.");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Shrieker {
    const VST3_CLASS_ID: [u8; 16] = *b"ShriekerDanielMa";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nice_export_clap!(Shrieker);
nice_export_vst3!(Shrieker);
