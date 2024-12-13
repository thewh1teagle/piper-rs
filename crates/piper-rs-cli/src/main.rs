/*
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx.json

cargo install piper-rs-cli
piper-rs-cli en_US-hfc_female-medium.onnx.json "Hello from piper-rs-cli!"
*/

use clap::Parser;
use eyre::{bail, Result};
use piper_rs::synth::PiperSpeechSynthesizer;
use rodio::buffer::SamplesBuffer;
use std::path::{Path, PathBuf};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to JSON config
    config: String,

    /// Text to create
    text: String,

    /// Wav file path to create
    /// Optional, it will play it directly if not provided.
    #[arg(short, long)]
    out: Option<String>,

    /// Path to Model
    #[arg(short, long)]
    model: Option<String>,

    /// Path to Model
    #[arg(long)]
    speaker_id: Option<i64>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let model = args
        .model
        .map(|m| PathBuf::from(m))
        .unwrap_or_else(|| PathBuf::from(&args.config.replace(".onnx.json", ".onnx")));
    if !model.exists() {
        bail!("Model not found at {}", model.display())
    }
    let model = piper_rs::from_config_path(Path::new(&args.config)).unwrap();
    if let Some(sid) = args.speaker_id {
        model.set_speaker(sid);
    }
    let synth = PiperSpeechSynthesizer::new(model).unwrap();


    if let Some(path) = args.out {
        // Save to file
        synth
            .synthesize_to_file(&PathBuf::from(&path), args.text, None)?;
        println!("Created {}", path);
    } else {
        // Play directly in memory
        let audio = synth.synthesize_parallel(args.text, None).unwrap();
        let mut samples: Vec<f32> = Vec::new();
        for result in audio {
            samples.append(&mut result.unwrap().into_vec());
        }
    
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
    
        let buf = SamplesBuffer::new(1, 22050, samples);
        sink.append(buf);
    
        println!("Playing...");
        sink.sleep_until_end();
    }
    Ok(())
}
