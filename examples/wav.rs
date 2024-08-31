/*
git submodule update --init
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx.json

cargo run --example wav en_US-hfc_female-medium.onnx.json output.wav
*/

use piper_rs::synth::SonataSpeechSynthesizer;
use std::path::Path;

fn init_ort_environment() {
    ort::init()
        .with_name("piper-rs")
        .commit()
        .expect("Failed to initialize onnxruntime");
}

fn main() {
    init_ort_environment();
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let output_path = std::env::args().nth(2).expect("Please specify output path");
    let text = "Hello! this is example with sonata".to_string();
    let voice = piper_rs::from_config_path(Path::new(&config_path)).unwrap();
    let synth = SonataSpeechSynthesizer::new(voice).unwrap();
    synth
        .synthesize_to_file(Path::new(&output_path), text, None)
        .unwrap();
}
