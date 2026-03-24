use std::collections::HashMap;

use ndarray::{Array1, Array2};
use ort::session::{Session, SessionInputValue, SessionInputs};
use ort::value::Value;
use serde::Deserialize;

use crate::PiperError;
use crate::PiperResult;

pub const BOS: char = '^';
pub const EOS: char = '$';
pub const PAD: char = '_';

#[derive(Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
}

#[derive(Deserialize)]
pub struct ESpeakConfig {
    pub voice: String,
}

#[derive(Deserialize, Clone)]
pub struct InferenceConfig {
    pub noise_scale: f32,
    pub length_scale: f32,
    pub noise_w: f32,
}

#[derive(Deserialize)]
pub struct ModelConfig {
    pub audio: AudioConfig,
    pub espeak: ESpeakConfig,
    pub inference: InferenceConfig,
    pub num_speakers: u32,
    pub speaker_id_map: HashMap<String, i64>,
    pub phoneme_id_map: HashMap<char, Vec<i64>>,
}

pub fn phonemes_to_ids(config: &ModelConfig, phonemes: &str) -> Vec<i64> {
    let map = &config.phoneme_id_map;
    let pad_id = *map.get(&PAD).and_then(|v| v.first()).unwrap_or(&0);
    let bos_id = *map.get(&BOS).and_then(|v| v.first()).unwrap_or(&0);
    let eos_id = *map.get(&EOS).and_then(|v| v.first()).unwrap_or(&0);

    let mut ids = Vec::with_capacity((phonemes.len() + 1) * 2);
    ids.push(bos_id);
    for ch in phonemes.chars() {
        if let Some(id) = map.get(&ch).and_then(|v| v.first()) {
            ids.push(*id);
            ids.push(pad_id);
        }
    }
    ids.push(eos_id);
    ids
}

pub fn infer(
    session: &Session,
    config: &ModelConfig,
    phonemes: &str,
    noise_scale: f32,
    length_scale: f32,
    noise_w: f32,
    speaker_id: i64,
) -> PiperResult<Vec<f32>> {
    let ids = phonemes_to_ids(config, phonemes);
    let input_len = ids.len();
    let input = Array2::<i64>::from_shape_vec((1, input_len), ids).unwrap();
    let input_lengths = Array1::<i64>::from_iter([input_len as i64]);
    let scales = Array1::<f32>::from_iter([noise_scale, length_scale, noise_w]);

    let mut inputs = vec![
        SessionInputValue::from(Value::from_array(input).unwrap()),
        SessionInputValue::from(Value::from_array(input_lengths).unwrap()),
        SessionInputValue::from(Value::from_array(scales).unwrap()),
    ];
    if config.num_speakers > 1 {
        let sid = Array1::<i64>::from_iter([speaker_id]);
        inputs.push(SessionInputValue::from(Value::from_array(sid).unwrap()));
    }

    let outputs = session
        .run(SessionInputs::from(inputs.as_slice()))
        .map_err(|e| PiperError::InferenceError(format!("Inference failed: {}", e)))?;

    let audio = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| PiperError::InferenceError(format!("Failed to extract output: {}", e)))?;

    Ok(audio.view().as_slice().unwrap().to_vec())
}
