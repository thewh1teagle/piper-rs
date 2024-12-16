/*
git submodule update --init
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx.json
cargo run --example enumerate_speakers en_US-libritts_r-medium.onnx.json
*/

use std::path::Path;

fn main() {
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let model = piper_rs::from_config_path(Path::new(&config_path)).unwrap();

    // Collect speakers into a Vec and sort them by name
    let mut speakers = model
        .get_speakers()
        .unwrap()
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    speakers.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by name (the second element of the tuple)

    // Print the sorted speakers
    for (id, name) in speakers.iter() {
        println!("ID: {:<3}  Label: {}", id, name);
    }
    println!("Found {} speakers.", speakers.len());
}
