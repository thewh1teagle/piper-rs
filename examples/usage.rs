/*
git submodule update --init

** You can specify speaker id in the last argumnet

wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx.json
cargo run --example usage en_US-libritts_r-medium.onnx.json 80
*/

use piper_rs::synth::PiperSpeechSynthesizer;
use rodio::buffer::SamplesBuffer;
use std::path::Path;

fn main() {
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let text = "Hello! i'm playing audio from memory directly with piper-rs.".to_string();
    let sid = std::env::args().nth(2);

    let model = piper_rs::from_config_path(Path::new(&config_path)).unwrap();
    // Set speaker ID
    if let Some(sid) = sid {
        let sid = sid.parse::<i64>().expect("Speaker ID should be number!");
        model.set_speaker(sid);
    }
    let synth = PiperSpeechSynthesizer::new(model).unwrap();
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
