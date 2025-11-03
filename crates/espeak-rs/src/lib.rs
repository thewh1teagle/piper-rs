use espeak_rs_sys;
use ffi_support::{rust_string_to_c, FfiStr};
use once_cell::sync::Lazy;
use regex::Regex;
use std::env;
use std::error::Error;
use std::ffi;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use unicode_normalization::UnicodeNormalization;

pub type ESpeakResult<T> = Result<T, ESpeakError>;

const CLAUSE_INTONATION_FULL_STOP: i32 = 0x00000000;
const CLAUSE_INTONATION_COMMA: i32 = 0x00001000;
const CLAUSE_INTONATION_QUESTION: i32 = 0x00002000;
const CLAUSE_INTONATION_EXCLAMATION: i32 = 0x00003000;
const CLAUSE_TYPE_SENTENCE: i32 = 0x00080000;
/// Name of the environment variable that points to the directory that contains `espeak-ng-data` directory
/// only needed if `espeak-ng-data` directory is not in the expected location (i.e. eSpeak-ng is not installed system wide)
const PIPER_ESPEAKNG_DATA_DIRECTORY: &str = "PIPER_ESPEAKNG_DATA_DIRECTORY";
const ESPEAKNG_DATA_DIR_NAME: &str = "espeak-ng-data";

#[derive(Debug, Clone)]
pub struct ESpeakError(pub String);

impl Error for ESpeakError {}

impl fmt::Display for ESpeakError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "eSpeak-ng Error :{}", self.0)
    }
}

static LANG_SWITCH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\([^)]*\)").unwrap());
static STRESS_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ˈˌ]").unwrap());
static ESPEAKNG_INIT: Lazy<ESpeakResult<()>> = Lazy::new(|| {
    let espeak_data_location = match env::var(PIPER_ESPEAKNG_DATA_DIRECTORY) {
        Ok(env_dir) => PathBuf::from(env_dir), // 1. From PIPER_ESPEAKNG_DATA_DIRECTORY environment variable
        Err(_) => {
            // 2. From the current working directory (CWD)
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if cwd.join(ESPEAKNG_DATA_DIR_NAME).exists() {
                cwd
            } else {
                // 3. From the parent directory of the current executable
                env::current_exe()
                    .unwrap_or_else(|_| PathBuf::new())
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            }
        }
    };
    let es_data_path_ptr = if espeak_data_location.join(ESPEAKNG_DATA_DIR_NAME).exists() {
        rust_string_to_c(espeak_data_location.display().to_string())
    } else {
        std::ptr::null()
    };
    unsafe {
        let es_sample_rate = espeak_rs_sys::espeak_Initialize(
            espeak_rs_sys::espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL,
            0,
            es_data_path_ptr,
            espeak_rs_sys::espeakINITIALIZE_DONT_EXIT as i32,
        );
        if es_sample_rate <= 0 {
            Err(ESpeakError(format!(
                "Failed to initialize eSpeak-ng. Try setting `{PIPER_ESPEAKNG_DATA_DIRECTORY}` environment variable to the directory that contains the `{ESPEAKNG_DATA_DIR_NAME}` directory. \
                Error code: `{es_sample_rate}`."
            )))
        } else {
            Ok(())
        }
    }
});

pub fn text_to_phonemes(
    text: &str,
    language: &str,
    phoneme_separator: Option<char>,
    remove_lang_switch_flags: bool,
    remove_stress: bool,
) -> ESpeakResult<Vec<String>> {
    let mut phonemes = Vec::new();
    for line in text.lines() {
        phonemes.append(&mut _text_to_phonemes(
            line,
            language,
            phoneme_separator,
            remove_lang_switch_flags,
            remove_stress,
        )?)
    }
    Ok(phonemes)
}

pub fn _text_to_phonemes(
    text: &str,
    language: &str,
    phoneme_separator: Option<char>,
    remove_lang_switch_flags: bool,
    remove_stress: bool,
) -> ESpeakResult<Vec<String>> {
    if let Err(ref e) = Lazy::force(&ESPEAKNG_INIT) {
        return Err(e.clone());
    }
    let set_voice_res = unsafe { espeak_rs_sys::espeak_SetVoiceByName(rust_string_to_c(language)) };
    if set_voice_res != espeak_rs_sys::espeak_ERROR_EE_OK {
        return Err(ESpeakError(format!(
            "Failed to set eSpeak-ng voice to: `{}` ",
            language
        )));
    }
    let calculated_phoneme_mode = match phoneme_separator {
        Some(c) => ((c as u32) << 8u32) | espeak_rs_sys::espeakINITIALIZE_PHONEME_IPA,
        None => espeak_rs_sys::espeakINITIALIZE_PHONEME_IPA,
    };
    let phoneme_mode: i32 = calculated_phoneme_mode.try_into().unwrap();
    let mut sent_phonemes = Vec::new();
    let mut phonemes = String::new();
    let mut text_c_char = rust_string_to_c(text) as *const ffi::c_char;
    let text_c_char_ptr = std::ptr::addr_of_mut!(text_c_char);
    let terminator: ffi::c_int = 0;
    while !text_c_char.is_null() {
        let ph_str = unsafe {
            let res = espeak_rs_sys::espeak_TextToPhonemes(
                text_c_char_ptr as _,
                espeak_rs_sys::espeakCHARS_UTF8.try_into().unwrap(),
                phoneme_mode,
            );
            FfiStr::from_raw(res)
        };

        // Decompose phonemes into UTF-8 codepoints.
        // This separates accent characters into separate "phonemes".
        // This solves issues with combined characters like "ç" (c with cedilla) being treated as a single phoneme (as it is generated by eSpeak-ng for German and other languages).
        let ph_string_composed = ph_str.into_string();
        let ph_string_decomposed = ph_string_composed.chars().nfd().collect::<String>();

        phonemes.push_str(&ph_string_decomposed);
        let intonation = terminator & 0x0000F000;
        if intonation == CLAUSE_INTONATION_FULL_STOP {
            phonemes.push('.');
        } else if intonation == CLAUSE_INTONATION_COMMA {
            phonemes.push(',');
        } else if intonation == CLAUSE_INTONATION_QUESTION {
            phonemes.push('?');
        } else if intonation == CLAUSE_INTONATION_EXCLAMATION {
            phonemes.push('!');
        }
        if (terminator & CLAUSE_TYPE_SENTENCE) == CLAUSE_TYPE_SENTENCE {
            sent_phonemes.push(std::mem::take(&mut phonemes));
        }
    }
    if !phonemes.is_empty() {
        sent_phonemes.push(std::mem::take(&mut phonemes));
    }
    if remove_lang_switch_flags {
        sent_phonemes = Vec::from_iter(
            sent_phonemes
                .into_iter()
                .map(|sent| LANG_SWITCH_PATTERN.replace_all(&sent, "").into_owned()),
        );
    }
    if remove_stress {
        sent_phonemes = Vec::from_iter(
            sent_phonemes
                .into_iter()
                .map(|sent| STRESS_PATTERN.replace_all(&sent, "").into_owned()),
        );
    }
    Ok(sent_phonemes)
}

// ==============================

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT_ALICE: &str =
        "Who are you? said the Caterpillar. Replied Alice , rather shyly, I hardly know, sir!";

    #[test]
    fn test_basic_en() -> ESpeakResult<()> {
        let text = "test";
        let expected = "tˈɛst.";
        let phonemes = text_to_phonemes(text, "en-US", None, false, false)?.join("");
        assert_eq!(phonemes, expected);
        Ok(())
    }

    #[test]
    fn test_it_splits_sentences() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes(TEXT_ALICE, "en-US", None, false, false)?;
        assert_eq!(phonemes.len(), 3);
        Ok(())
    }

    #[test]
    fn test_it_adds_phoneme_separator() -> ESpeakResult<()> {
        let text = "test";
        let expected = "t_ˈɛ_s_t.";
        let phonemes = text_to_phonemes(text, "en-US", Some('_'), false, false)
            .unwrap()
            .join("");
        assert_eq!(phonemes, expected);
        Ok(())
    }

    #[test]
    fn test_it_preserves_clause_breakers() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes(TEXT_ALICE, "en-US", None, false, false)?.join("");
        let clause_breakers = ['.', ',', '?', '!'];
        for c in clause_breakers {
            assert_eq!(
                phonemes.contains(c),
                true,
                "Clause breaker `{}` not preserved",
                c
            );
        }
        Ok(())
    }

    #[test]
    fn test_arabic() -> ESpeakResult<()> {
        let text = "مَرْحَبَاً بِكَ أَيُّهَا الْرَّجُلْ";
        let expected = "mˈarħabˌaː bikˌa ʔaˈiːuhˌaː alrrˈadʒul.";
        let phonemes = text_to_phonemes(text, "ar", None, false, false)?.join("");
        assert_eq!(phonemes, expected);
        Ok(())
    }

    #[test]
    fn test_lang_switch_flags() -> ESpeakResult<()> {
        let text = "Hello معناها مرحباً";

        let with_lang_switch = text_to_phonemes(text, "ar", None, false, false)?.join("");
        assert_eq!(with_lang_switch.contains("(en)"), true);
        assert_eq!(with_lang_switch.contains("(ar)"), true);

        let without_lang_switch = text_to_phonemes(text, "ar", None, true, false)?.join("");
        assert_eq!(without_lang_switch.contains("(en)"), false);
        assert_eq!(without_lang_switch.contains("(ar)"), false);

        Ok(())
    }

    #[test]
    fn test_stress() -> ESpeakResult<()> {
        let stress_markers = ['ˈ', 'ˌ'];

        let with_stress = text_to_phonemes(TEXT_ALICE, "en-US", None, false, false)?.join("");
        assert_eq!(with_stress.contains(stress_markers), true);

        let without_stress = text_to_phonemes(TEXT_ALICE, "en-US", None, false, true)?.join("");
        assert_eq!(without_stress.contains(stress_markers), false);

        Ok(())
    }
    #[test]
    fn test_line_splitting() -> ESpeakResult<()> {
        let text = "Hello\nThere\nAnd\nWelcome";
        let phoneme_paragraphs = text_to_phonemes(text, "en-US", None, false, false)?;
        assert_eq!(phoneme_paragraphs.len(), 4);
        Ok(())
    }
}
