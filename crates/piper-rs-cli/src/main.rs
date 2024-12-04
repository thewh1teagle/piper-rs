/*
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx.json

cargo install piper-rs-cli
piper-rs-cli en_US-hfc_female-medium.onnx.json "Hello from piper-rs-cli!"
*/

use clap::Parser;
use eyre::{bail, Result};
use piper_rs::synth::SonataSpeechSynthesizer;
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
    out_path: Option<String>,

    /// Path to Model
    #[arg(short, long)]
    model: Option<String>,
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
    let voice = piper_rs::from_config_path(Path::new(&args.config)).unwrap();
    let synth = SonataSpeechSynthesizer::new(voice).unwrap();

    if let Some(path) = args.out_path {
        // Save to file
        synth
            .synthesize_to_file(&PathBuf::from(path), args.text, None)?;
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
    
        sink.sleep_until_end();
    }
    Ok(())
}
