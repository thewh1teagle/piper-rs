/*
git submodule update --init
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx.json

cargo run --example multi en_US-libritts_r-medium.onnx.json 1
*/

use core::panic;
use std::path::Path;

use piper_rs::synth::SonataSpeechSynthesizer;
use rodio::buffer::SamplesBuffer;

fn init_ort_environment() {
    ort::init()
        .with_name("piper-rs")
        .commit()
        .expect("Failed to initialize onnxruntime");
}

fn main() {
    init_ort_environment();
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let sid = std::env::args()
        .nth(2)
        .expect("Please specify speaker id")
        .parse()
        .unwrap();

    let text = "Hello! I'm playing audio from memory directly with piper-rs.".to_string();

    let model = piper_rs::from_config_path(Path::new(&config_path)).unwrap();

    println!(
        "Number of speakers: {}",
        model.get_speakers().unwrap().unwrap().keys().len()
    );
    println!("Using speaker: {}", sid);

    if let Some(err) = model.set_speaker(&sid) {
        panic!("{}", err)
    }

    let synth = SonataSpeechSynthesizer::new(model).unwrap();

    let mut samples: Vec<f32> = Vec::new();
    let audio = synth.synthesize_parallel(text, None).unwrap();
    for result in audio {
        samples.append(&mut result.unwrap().into_vec());
    }

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let buf = SamplesBuffer::new(1, 22050, samples);
    sink.append(buf);

    sink.sleep_until_end();
}
