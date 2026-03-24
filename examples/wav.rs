/*
git submodule update --init

wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/libritts_r/medium/en_US-libritts_r-medium.onnx.json
cargo run --example wav en_US-libritts_r-medium.onnx.json output.wav 50
*/

use piper_rs::Piper;
use std::io::Write;
use std::path::Path;

fn main() {
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let output_path = std::env::args().nth(2).expect("Please specify output path");
    let speaker_id: Option<i64> = std::env::args().nth(3).map(|s| s.parse().expect("Speaker ID must be a number"));

    let onnx_path = config_path.replace(".onnx.json", ".onnx");
    let piper = Piper::new(Path::new(&onnx_path), Path::new(&config_path)).unwrap();

    let text = "Hello! This file was created by piper-rs.";
    let (samples, sample_rate) = piper.create(text, false, speaker_id, None, None, None).unwrap();

    // Convert f32 samples to i16 PCM and write a WAV file
    let samples_i16: Vec<i16> = samples.iter().map(|&s| (s * i16::MAX as f32) as i16).collect();
    let mut file = std::fs::File::create(&output_path).unwrap();
    write_wav(&mut file, &samples_i16, sample_rate, 1);
    println!("Saved to {}", output_path);
}

fn write_wav(w: &mut impl Write, samples: &[i16], sample_rate: u32, channels: u16) {
    let data_len = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * channels as u32 * 2;
    w.write_all(b"RIFF").unwrap();
    w.write_all(&(36 + data_len).to_le_bytes()).unwrap();
    w.write_all(b"WAVEfmt ").unwrap();
    w.write_all(&16u32.to_le_bytes()).unwrap();
    w.write_all(&1u16.to_le_bytes()).unwrap();
    w.write_all(&channels.to_le_bytes()).unwrap();
    w.write_all(&sample_rate.to_le_bytes()).unwrap();
    w.write_all(&byte_rate.to_le_bytes()).unwrap();
    w.write_all(&(channels * 2).to_le_bytes()).unwrap();
    w.write_all(&16u16.to_le_bytes()).unwrap();
    w.write_all(b"data").unwrap();
    w.write_all(&data_len.to_le_bytes()).unwrap();
    for &s in samples {
        w.write_all(&s.to_le_bytes()).unwrap();
    }
}
