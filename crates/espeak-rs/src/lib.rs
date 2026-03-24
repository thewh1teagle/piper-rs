use std::env;
use std::ffi::{c_char, c_void, CStr, CString};
use std::mem;
use std::path::PathBuf;
use std::ptr;
use std::sync::OnceLock;

const PIPER_ESPEAKNG_DATA_DIRECTORY: &str = "PIPER_ESPEAKNG_DATA_DIRECTORY";
const ESPEAKNG_DATA_DIR_NAME: &str = "espeak-ng-data";

#[derive(Debug, Clone)]
pub struct ESpeakError(pub String);

impl std::error::Error for ESpeakError {}

impl std::fmt::Display for ESpeakError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "eSpeak-ng error: {}", self.0)
    }
}

pub type ESpeakResult<T> = Result<T, ESpeakError>;

static ESPEAK_INIT: OnceLock<ESpeakResult<()>> = OnceLock::new();

fn init_espeak() -> ESpeakResult<()> {
    let data_dir = locate_espeak_data();
    // Keep the CString alive until after Initialize returns.
    let path_cstr = data_dir
        .as_ref()
        .and_then(|p| CString::new(p.to_string_lossy().as_ref()).ok());
    let path_ptr = path_cstr.as_ref().map_or(ptr::null(), |c| c.as_ptr());

    let sample_rate = unsafe {
        espeak_rs_sys::espeak_Initialize(
            espeak_rs_sys::espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL,
            0,
            path_ptr,
            espeak_rs_sys::espeakINITIALIZE_DONT_EXIT as i32,
        )
    };

    if sample_rate <= 0 {
        Err(ESpeakError(format!(
            "Failed to initialize eSpeak-ng (code {sample_rate}). \
            Try setting `{PIPER_ESPEAKNG_DATA_DIRECTORY}` to the directory containing `{ESPEAKNG_DATA_DIR_NAME}`."
        )))
    } else {
        Ok(())
    }
}

fn locate_espeak_data() -> Option<PathBuf> {
    // 1. Environment variable
    if let Ok(dir) = env::var(PIPER_ESPEAKNG_DATA_DIRECTORY) {
        let p = PathBuf::from(dir);
        if p.join(ESPEAKNG_DATA_DIR_NAME).exists() {
            return Some(p);
        }
    }
    // 2. Current working directory
    if let Ok(cwd) = env::current_dir() {
        if cwd.join(ESPEAKNG_DATA_DIR_NAME).exists() {
            return Some(cwd);
        }
    }
    // 3. Directory of the current executable
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join(ESPEAKNG_DATA_DIR_NAME).exists() {
                return Some(dir.to_path_buf());
            }
        }
    }
    None
}

/// Strip inline language-switch markers of the form `(xx)` that espeak inserts
/// when the text contains words from a different language than the active voice.
fn strip_lang_switches(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth: usize = 0;
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Convert `text` to IPA phonemes using the given espeak-ng voice/language.
///
/// `espeak_TextToPhonemes` returns one clause at a time (advancing an internal
/// pointer through the input). Clauses that end a sentence are terminated by
/// `.`, `?`, or `!` in the phoneme output; sub-clauses (comma, semicolon, …)
/// end with the corresponding punctuation but do not break a sentence.
/// This function accumulates sub-clauses and emits one `String` per sentence.
///
/// Inline language-switch markers (`(en)`, `(ar)`, …) are always stripped.
pub fn text_to_phonemes(
    text: &str,
    language: &str,
    phoneme_separator: Option<char>,
) -> ESpeakResult<Vec<String>> {
    // Ensure the library is initialised exactly once.
    ESPEAK_INIT
        .get_or_init(init_espeak)
        .as_ref()
        .map_err(|e| e.clone())?;

    let lang_cstr = CString::new(language)
        .map_err(|_| ESpeakError("Language name contains a null byte".into()))?;
    let set_voice = unsafe { espeak_rs_sys::espeak_SetVoiceByName(lang_cstr.as_ptr()) };
    if set_voice != espeak_rs_sys::espeak_ERROR_EE_OK {
        return Err(ESpeakError(format!("Failed to set voice: `{language}`")));
    }

    let phoneme_mode = match phoneme_separator {
        Some(c) => ((c as u32) << 8) | espeak_rs_sys::espeakINITIALIZE_PHONEME_IPA,
        None => espeak_rs_sys::espeakINITIALIZE_PHONEME_IPA,
    } as i32;

    let mut sentences: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        let text_cstr = CString::new(line)
            .map_err(|_| ESpeakError("Text contains a null byte".into()))?;

        // espeak advances this pointer clause by clause, setting it to null when done.
        let mut text_ptr: *const c_char = text_cstr.as_ptr();

        while !text_ptr.is_null() {
            let clause = unsafe {
                let res = espeak_rs_sys::espeak_TextToPhonemes(
                    &mut text_ptr as *mut *const c_char as *mut *const c_void,
                    espeak_rs_sys::espeakCHARS_UTF8 as i32,
                    phoneme_mode,
                );
                if res.is_null() {
                    continue;
                }
                CStr::from_ptr(res).to_string_lossy().into_owned()
            };

            let clause = strip_lang_switches(&clause);
            if clause.is_empty() {
                continue;
            }

            current.push_str(&clause);

            // espeak appends the clause-ending punctuation to the phoneme string.
            // A sentence boundary is '.', '?', or '!'; commas etc. are sub-clauses.
            if matches!(current.trim_end().chars().last(), Some('.' | '?' | '!')) {
                sentences.push(mem::take(&mut current));
            }
        }

        // Flush any trailing content that didn't end with sentence punctuation.
        if !current.is_empty() {
            sentences.push(mem::take(&mut current));
        }
    }

    Ok(sentences)
}

// ==============================

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT_ALICE: &str =
        "Who are you? said the Caterpillar. Replied Alice , rather shyly, I hardly know, sir!";

    #[test]
    fn test_basic_en() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes("test", "en-US", None)?.join("");
        assert_eq!(phonemes, "tˈɛst.");
        Ok(())
    }

    #[test]
    fn test_it_splits_sentences() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes(TEXT_ALICE, "en-US", None)?;
        assert_eq!(phonemes.len(), 3);
        Ok(())
    }

    #[test]
    fn test_it_adds_phoneme_separator() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes("test", "en-US", Some('_'))?.join("");
        assert_eq!(phonemes, "t_ˈɛ_s_t.");
        Ok(())
    }

    #[test]
    fn test_it_preserves_clause_breakers() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes(TEXT_ALICE, "en-US", None)?.join("");
        for c in ['.', ',', '?', '!'] {
            assert!(phonemes.contains(c), "Clause breaker `{c}` not preserved");
        }
        Ok(())
    }

    #[test]
    fn test_arabic() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes("مَرْحَبَاً بِكَ أَيُّهَا الْرَّجُلْ", "ar", None)?.join("");
        assert_eq!(phonemes, "mˈarħabˌaː bikˌa ʔaˈiːuhˌaː alrrˈadʒul.");
        Ok(())
    }

    #[test]
    fn test_lang_switch_markers_stripped() -> ESpeakResult<()> {
        // Mixed-language text: espeak inserts (en)/(ar) markers; we always strip them.
        let phonemes = text_to_phonemes("Hello معناها مرحباً", "ar", None)?.join("");
        assert!(!phonemes.contains("(en)"));
        assert!(!phonemes.contains("(ar)"));
        Ok(())
    }

    #[test]
    fn test_line_splitting() -> ESpeakResult<()> {
        let phonemes = text_to_phonemes("Hello\nThere\nAnd\nWelcome", "en-US", None)?;
        assert_eq!(phonemes.len(), 4);
        Ok(())
    }
}
