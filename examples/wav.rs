/*
git submodule update --init

wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx.json
cargo run --example wav en_US-libritts_r-medium.onnx.json output.wav 50
*/

use piper_rs::synth::PiperSpeechSynthesizer;
use std::path::Path;

fn main() {
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let output_path = std::env::args().nth(2).expect("Please specify output path");
    let sid = std::env::args().nth(3);
    let text = "Hello! this file created by piper-rs.".to_string();
    let model = piper_rs::from_config_path(Path::new(&config_path)).unwrap();
    // Set speaker ID
    if let Some(sid) = sid {
        let sid = sid.parse::<i64>().expect("Speaker ID should be number!");
        model.set_speaker(sid);
    }
    let synth = PiperSpeechSynthesizer::new(model).unwrap();
    synth
        .synthesize_to_file(Path::new(&output_path), text, None)
        .unwrap();
}
