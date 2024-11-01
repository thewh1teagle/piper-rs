/*
git submodule update --init

Female:
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx.json
cargo run --example usage en_US-hfc_female-medium.onnx.json

Male:
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/hfc_male/medium/en_US-hfc_male-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/hfc_male/medium/en_US-hfc_male-medium.onnx.json
cargo run --example usage en_US-hfc_male-medium.onnx.json

See full list of TTS models at:
https://huggingface.co/rhasspy/piper-voices/tree/main
*/

use piper_rs::synth::SonataSpeechSynthesizer;
use rodio::buffer::SamplesBuffer;
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
    let text = "Hello! i'm playing audio from memory directly with piper-rs.".to_string();

    let voice = piper_rs::from_config_path(Path::new(&config_path)).unwrap();
    let synth = SonataSpeechSynthesizer::new(voice).unwrap();
    let mut samples: Vec<f32> = Vec::new();
    let audio = synth.synthesize_parallel(text, None).unwrap();
    for result in audio {
        samples.append(&mut result.unwrap().into_vec());
    }

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    sink.set_volume(4.0);

    let buf = SamplesBuffer::new(1, 22050, samples);
    sink.append(buf);
    sink.sleep_until_end();
}
