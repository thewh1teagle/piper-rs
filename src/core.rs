use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

pub use crate::audio_ops::{Audio, AudioInfo, AudioSamples, WaveWriterError};

pub type PiperResult<T> = Result<T, PiperError>;
pub type PiperAudioResult = PiperResult<Audio>;
pub type AudioStreamIterator<'a> =
    Box<dyn Iterator<Item = PiperResult<AudioSamples>> + Send + Sync + 'a>;

#[derive(Debug)]
pub enum PiperError {
    FailedToLoadResource(String),
    PhonemizationError(String),
    OperationError(String),
}

impl PiperError {
    pub fn with_message(message: impl Into<String>) -> Self {
        Self::OperationError(message.into())
    }
}
impl Error for PiperError {}

impl fmt::Display for PiperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_message = match self {
            PiperError::FailedToLoadResource(msg) => {
                format!("Failed to load resource from. Error `{}`", msg)
            }
            PiperError::PhonemizationError(msg) => msg.to_string(),
            PiperError::OperationError(msg) => msg.to_string(),
        };
        write!(f, "{}", err_message)
    }
}

impl From<WaveWriterError> for PiperError {
    fn from(error: WaveWriterError) -> Self {
        PiperError::OperationError(error.to_string())
    }
}

/// A wrapper type that holds sentence phonemes
pub struct Phonemes(Vec<String>);

impl Phonemes {
    pub fn sentences(&self) -> &Vec<String> {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.0.clone()
    }

    pub fn num_sentences(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<String>> for Phonemes {
    fn from(other: Vec<String>) -> Self {
        Self(other)
    }
}

#[allow(clippy::to_string_trait_impl)]
impl std::string::ToString for Phonemes {
    fn to_string(&self) -> String {
        self.0.join(" ")
    }
}

pub trait PiperModel {
    fn audio_output_info(&self) -> PiperResult<AudioInfo>;
    fn phonemize_text(&self, text: &str) -> PiperResult<Phonemes>;
    fn speak_batch(&self, phoneme_batches: Vec<String>) -> PiperResult<Vec<Audio>>;
    fn speak_one_sentence(&self, phonemes: String) -> PiperAudioResult;

    fn get_default_synthesis_config(&self) -> PiperResult<Box<dyn Any>>;
    fn get_fallback_synthesis_config(&self) -> PiperResult<Box<dyn Any>>;
    fn set_fallback_synthesis_config(&self, synthesis_config: &dyn Any) -> PiperResult<()>;

    fn get_language(&self) -> PiperResult<Option<String>> {
        Ok(None)
    }
    fn get_speakers(&self) -> PiperResult<Option<&HashMap<i64, String>>> {
        Ok(None)
    }
    fn set_speaker(&self, sid: i64) -> Option<PiperError>;
    fn speaker_id_to_name(&self, sid: i64) -> PiperResult<Option<String>> {
        Ok(self
            .get_speakers()?
            .and_then(|speakers| speakers.get(&sid))
            .cloned())
    }
    fn speaker_name_to_id(&self, name: &str) -> PiperResult<Option<i64>> {
        Ok(self.get_speakers()?.and_then(|speakers| {
            for (sid, sname) in speakers {
                if sname == name {
                    return Some(*sid);
                }
            }
            None
        }))
    }
    fn properties(&self) -> PiperResult<HashMap<String, String>> {
        Ok(HashMap::with_capacity(0))
    }

    fn supports_streaming_output(&self) -> bool {
        false
    }
    fn stream_synthesis(
        &self,
        #[allow(unused_variables)] phonemes: String,
        #[allow(unused_variables)] chunk_size: usize,
        #[allow(unused_variables)] chunk_padding: usize,
    ) -> PiperResult<AudioStreamIterator> {
        Err(PiperError::OperationError(
            "Streaming synthesis is not supported for this model".to_string(),
        ))
    }
}
