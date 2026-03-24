/*
git submodule update --init

wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx.json
cargo run --example enumerate_speakers en_US-libritts_r-medium.onnx.json
*/

use piper_rs::Piper;
use std::path::Path;

fn main() {
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let onnx_path = config_path.replace(".onnx.json", ".onnx");
    let piper = Piper::new(Path::new(&onnx_path), Path::new(&config_path)).unwrap();

    match piper.voices() {
        None => println!("Single-speaker model."),
        Some(voices) => {
            let mut speakers: Vec<_> = voices.iter().collect();
            speakers.sort_by_key(|(name, _)| name.as_str());
            for (name, id) in &speakers {
                println!("ID: {:<3}  Name: {}", id, name);
            }
            println!("Found {} speakers.", speakers.len());
        }
    }
}
