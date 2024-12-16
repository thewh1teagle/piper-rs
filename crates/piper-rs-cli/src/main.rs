/*
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx.json

cargo run -p piper-rs-cli en_US-hfc_female-medium.onnx.json "Hello from piper-rs!"
*/

use clap::Parser;
use console::style;
use eyre::{bail, Result};
use piper_rs::synth::PiperSpeechSynthesizer;
use rodio::buffer::SamplesBuffer;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

    #[arg(long)]
    verbose: bool,
}

fn show_error_hint() {
    eprintln!(
        "\n{}{}\n{}",
        style("Hint: You can download the required files with:\n").bold().blue(),
        style("wget https://github.com/thewh1teagle/piper-rs/releases/download/espeak-ng-files/espeak-ng-data.tar.gz\n\
            tar xf espeak-ng-data.tar.gz").bold().green().italic(),
        style("Make sure the folder is placed next to the executable, in the working directory, or set the PIPER_ESPEAKNG_DATA_DIRECTORY environment variable.")
            .bold()
            .blue() // Hint in blue
    );
}

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    tracing::debug!("hi");
    let args = Args::parse();
    match run(&args) {
        Ok(_) => {}
        Err(e) => {
            if args.verbose {
                eprintln!("{:?}", e);
            }
            show_error_hint();
        }
    }
}

fn run(args: &Args) -> Result<()> {
    let model = args
        .model
        .clone()
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

    if let Some(path) = &args.out {
        // Save to file
        let start_t = Instant::now();
        synth.synthesize_to_file(&PathBuf::from(&path), args.text.clone(), None)?;
        tracing::debug!("Took {} seconds", start_t.elapsed().as_secs());
        println!("Created {}", path);
    } else {
        // Play directly in memory
        let start_t = Instant::now();
        let audio = synth.synthesize_parallel(args.text.clone(), None)?;
        tracing::debug!("Took {} seconds", start_t.elapsed().as_secs());
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
