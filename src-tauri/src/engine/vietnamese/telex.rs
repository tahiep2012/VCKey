use std::sync::atomic::{AtomicBool, Ordering};
use crate::engine::vietnamese::unikey_ffi::{UnikeyEngine, InputMethod};
use crate::engine::vietnamese::english::is_english_word;

/// Runtime flag — mirrors AppSettings::vn_auto_english_restore.
/// When false, the English-word dictionary bypass is disabled.
pub static VN_AUTO_ENGLISH_RESTORE: AtomicBool = AtomicBool::new(true);

/// Checks if a string contains any Vietnamese accented vowels or the letter đ/Đ.
pub fn contains_vietnamese_accent(s: &str) -> bool {
    s.chars().any(|c| matches!(
        c,
        'á' | 'à' | 'ả' | 'ã' | 'ạ'
        | 'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ'
        | 'â' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ'
        | 'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ'
        | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ'
        | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị'
        | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ'
        | 'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ'
        | 'ơ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ'
        | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ'
        | 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự'
        | 'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ'
        | 'đ' | 'Á' | 'À' | 'Ả' | 'Ã' | 'Ạ'
        | 'Ă' | 'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ặ'
        | 'Â' | 'Ấ' | 'Ầ' | 'Ẩ' | 'Ẫ' | 'Ậ'
        | 'É' | 'È' | 'Ẻ' | 'Ẽ' | 'Ẹ'
        | 'Ê' | 'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ'
        | 'Í' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Ị'
        | 'Ó' | 'Ò' | 'Ỏ' | 'Õ' | 'Ọ'
        | 'Ô' | 'Ố' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ộ'
        | 'Ơ' | 'Ớ' | 'Ờ' | 'Ở' | 'Ỡ' | 'Ợ'
        | 'Ú' | 'Ù' | 'Ủ' | 'Ũ' | 'Ụ'
        | 'Ư' | 'Ứ' | 'Ừ' | 'Ử' | 'Ữ' | 'Ự'
        | 'Ý' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ỵ'
        | 'Đ'
    ))
}

pub struct VietnameseTelexEngine;

impl VietnameseTelexEngine {
    pub fn transform(raw: &str) -> String {
        Self::transform_with_method(raw, UnikeyEngine::get_input_method())
    }

    pub fn transform_with_method(raw: &str, method: InputMethod) -> String {
        if raw.is_empty() {
            return String::new();
        }

        match method {
            InputMethod::Vni => {
                UnikeyEngine::transform_with_method(raw, InputMethod::Vni)
            }
            InputMethod::Viqr => {
                UnikeyEngine::transform_with_method(raw, InputMethod::Viqr)
            }
            InputMethod::SimpleTelex | InputMethod::SimpleTelex2 => {
                UnikeyEngine::transform_with_method(raw, method)
            }
            InputMethod::Telex | _ => {
                let raw_lower = raw.to_lowercase();

                // 0. Standalone www / w / ww
                if raw_lower.starts_with("www") {
                    return raw.to_string();
                }
                if raw == "Ww" || raw == "WW" || raw == "wW" {
                    return "W".to_string();
                }
                if raw == "ww" {
                    return "w".to_string();
                }
                if raw == "W" {
                    return "Ư".to_string();
                }
                if raw == "w" {
                    return "ư".to_string();
                }

                // 1. Reorder tone modifiers placed inside final consonant clusters (e.g. cacsh -> cachs, cansh -> canhs, bansg -> bangs)
                let normalized = Self::reorder_interrupted_final_consonants(raw);
                let norm_lower = normalized.to_lowercase();

                // 2. Double roof escape (ooo -> oo, eee -> ee, aaa -> aa, ddd -> dd, e.g. goood -> good, queeen -> queen)
                let unroofed = if norm_lower.contains("ooo") || norm_lower.contains("eee") || norm_lower.contains("aaa") || norm_lower.contains("ddd") {
                    let mut s = normalized.clone();
                    s = s.replace("ooo", "oo").replace("OOO", "OO").replace("Ooo", "Oo")
                         .replace("eee", "ee").replace("EEE", "EE").replace("Eee", "Ee")
                         .replace("aaa", "aa").replace("AAA", "AA").replace("Aaa", "Aa")
                         .replace("ddd", "dd").replace("DDD", "DD").replace("Ddd", "Dd");
                    s
                } else {
                    normalized
                };

                // 3. Specific escape patterns (e.g. virffresh -> vifresh, ffresh -> fresh)
                if norm_lower.contains("ffresh") {
                    let s = unroofed.clone();
                    let replaced = s.replace("rffresh", "fresh").replace("RFFRESH", "FRESH").replace("Rffresh", "Fresh")
                                    .replace("ffresh", "fresh").replace("FFRESH", "FRESH").replace("Ffresh", "Fresh");
                    if is_english_word(&replaced.to_lowercase()) {
                        return replaced;
                    }
                    return UnikeyEngine::transform_with_method(&replaced, InputMethod::Telex);
                }

                // 4. Collapse double modifiers used to cancel tone/hat (e.g. Orrder -> Order, assus -> asus, Taurri -> Tauri, hiepsserver -> hiepserver)
                let collapsed = Self::collapse_double_modifiers(&unroofed);
                let collapsed_lower = collapsed.to_lowercase();

                // If double modifiers were collapsed (user intentionally typed duplicate modifier keys to cancel diacritics)
                if collapsed.len() < unroofed.len() {
                    return collapsed;
                }

                // 5. English dictionary bypass (only if auto-restore is enabled in settings)
                if VN_AUTO_ENGLISH_RESTORE.load(Ordering::Relaxed) && (is_english_word(&norm_lower) || is_english_word(&collapsed_lower)) {
                    return collapsed;
                }

                // 6. Double modifier cancellation at end (e.g. tess -> tes, teff -> tef, terr -> ter, showw -> show, boxx -> box)
                // Tone/horn cancellation (s, f, r, x, j, w) ONLY applies if the single modifier key
                // actually created a Vietnamese diacritic (e.g. "te" + 's' -> "tés", "bo" + 'x' -> "bõ").
                // If appending the single modifier did not create any diacritic (e.g. "relea" + 's' -> "releas", "serve" + 'r' -> "sẻver"),
                // it is NOT a tone cancellation and must be preserved as plain consonants.
                let double_modifier_cancellations = ["ss", "ff", "rr", "xx", "jj", "ww"];
                for dbl in double_modifier_cancellations {
                    if norm_lower.ends_with(dbl) && collapsed.len() >= 2 {
                        let prefix = &collapsed[..collapsed.len() - 2];
                        let single_mod = &collapsed[collapsed.len() - 1..];

                        // Special case for "ww": no English word ends with "ww", so "ww" at the end of a word
                        // is always intended to escape the horn/trăng (e.g. showw -> show, howw -> how, draww -> draw).
                        if *dbl == *"ww" {
                            let transformed_prefix = UnikeyEngine::transform_with_method(prefix, InputMethod::Telex);
                            return format!("{}{}", transformed_prefix, single_mod);
                        }

                        let prefix_with_single = format!("{}{}", prefix, single_mod);
                        let t_without = UnikeyEngine::transform_with_method(prefix, InputMethod::Telex);
                        let t_with = UnikeyEngine::transform_with_method(&prefix_with_single, InputMethod::Telex);
                        let literal_append = format!("{}{}", t_without, single_mod);

                        if t_with != literal_append {
                            let transformed_prefix = UnikeyEngine::transform_with_method(prefix, InputMethod::Telex);
                            return format!("{}{}", transformed_prefix, single_mod);
                        }
                    }
                }

                // 7. Double roof escape (ooo -> oo, eee -> ee, aaa -> aa, ddd -> dd)
                if norm_lower.contains("ooo") || norm_lower.contains("eee") || norm_lower.contains("aaa") || norm_lower.contains("ddd") {
                    let mut s = collapsed.clone();
                    s = s.replace("ooo", "oo").replace("OOO", "OO").replace("Ooo", "Oo")
                         .replace("eee", "ee").replace("EEE", "EE").replace("Eee", "Ee")
                         .replace("aaa", "aa").replace("AAA", "AA").replace("Aaa", "Aa")
                         .replace("ddd", "dd").replace("DDD", "DD").replace("Ddd", "Dd");
                    return s;
                }

                UnikeyEngine::transform_with_method(&collapsed, InputMethod::Telex)
            }
        }
    }

    /// Collapses duplicate tone modifier keys that were typed to cancel tones (e.g. "Orrder" -> "Order", "assus" -> "asus")
    /// while preserving valid English words that naturally have double letters (e.g. "sorry", "coffee", "pass", "class").
    pub fn collapse_double_modifiers(raw: &str) -> String {
        let raw_lower = raw.to_lowercase();
        if crate::engine::vietnamese::english::ENGLISH_DICT.contains(raw_lower.as_str()) {
            return raw.to_string();
        }

        // Collapse leading double-w (e.g. "wword" -> "word", "WWord" -> "Word", "WWo" -> "Wo")
        // while preserving "www"
        let s = if !raw_lower.starts_with("www") && raw.len() >= 2 {
            if raw.starts_with("WW") || raw.starts_with("Ww") || raw.starts_with("wW") {
                format!("W{}", &raw[2..])
            } else if raw.starts_with("ww") {
                format!("w{}", &raw[2..])
            } else {
                raw.to_string()
            }
        } else {
            raw.to_string()
        };

        let chars: Vec<char> = s.chars().collect();
        if chars.len() < 3 {
            return s;
        }

        let mut result = String::with_capacity(s.len());
        let mut i = 0;
        while i < chars.len() {
            let curr = chars[i].to_ascii_lowercase();
            let is_modifier = matches!(curr, 'r' | 's' | 'f' | 'x' | 'j' | 'w');
            let has_vowel_before = i >= 1 && chars[..i].iter().any(|&c| {
                matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
            });

            if is_modifier && has_vowel_before {
                let mut run_len = 0;
                while i + run_len < chars.len() && chars[i + run_len].to_ascii_lowercase() == curr {
                    run_len += 1;
                }
                if run_len >= 2 {
                    // Check if adding the first modifier key actually modified a diacritic/vowel.
                    // If t_with == t_without + curr, then the modifier was just treated as a literal consonant
                    // (e.g. "relea" + 's' -> "releas", "serve" + 'r' -> "sẻver").
                    // In that case, it did NOT create a diacritic, so the repeated key is NOT a cancellation!
                    let prefix: String = chars[..i].iter().collect();
                    let prefix_with_first: String = chars[..=i].iter().collect();
                    let t_without = UnikeyEngine::transform_with_method(&prefix, InputMethod::Telex);
                    let t_with = UnikeyEngine::transform_with_method(&prefix_with_first, InputMethod::Telex);
                    let literal_append = format!("{}{}", t_without, chars[i]);

                    let created_diacritic = t_with != literal_append;

                    if created_diacritic {
                        // For repeated modifier keys (e.g. rr, rrr, rrrr), the first pair cancels
                        // the diacritic and produces 1 character. Every subsequent keypress produces
                        // an additional character. Thus run_len keys yield run_len - 1 characters.
                        for j in 0..(run_len - 1) {
                            result.push(chars[i + j]);
                        }
                        i += run_len;
                        continue;
                    }
                }
            }
            result.push(chars[i]);
            i += 1;
        }
        result
    }

    /// Reorders tone modifiers typed in the middle of final consonant clusters
    /// (e.g. "cacsh" -> "cachs" -> "cách", "tinsh" -> "tinhs" -> "tính", "bansg" -> "bangs" -> "báng").
    /// In fast typing, users frequently press tone keys like 's', 'f', 'r', 'x', 'j' before completing final digraphs 'ch', 'nh', 'ng'.
    pub fn reorder_interrupted_final_consonants(raw: &str) -> String {
        let chars: Vec<char> = raw.chars().collect();
        if chars.len() < 3 {
            return raw.to_string();
        }

        let mut result: Vec<char> = Vec::with_capacity(chars.len());
        let mut i = 0;
        while i < chars.len() {
            if i + 2 < chars.len() {
                let c1 = chars[i].to_ascii_lowercase();
                let tone = chars[i + 1].to_ascii_lowercase();
                let c2 = chars[i + 2].to_ascii_lowercase();

                let is_tone = matches!(tone, 's' | 'f' | 'r' | 'x' | 'j');
                let is_final_digraph = (c1 == 'c' && c2 == 'h')
                    || (c1 == 'n' && c2 == 'h')
                    || (c1 == 'n' && c2 == 'g');

                // Ensure there is at least one vowel preceding c1
                let has_vowel_before = result.iter().any(|&prev| {
                    matches!(prev.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
                });

                if is_tone && is_final_digraph && has_vowel_before {
                    result.push(chars[i]);     // c or n
                    result.push(chars[i + 2]); // h or g
                    result.push(chars[i + 1]); // tone
                    i += 3;
                    continue;
                }
            }
            result.push(chars[i]);
            i += 1;
        }

        result.into_iter().collect()
    }

    fn remove_tones(s: &str) -> String {
        s.chars().map(|c| match c {
            'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
            'Á' | 'À' | 'Ả' | 'Ã' | 'Ạ' => 'A',
            'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'a',
            'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ặ' => 'A',
            'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'a',
            'Ấ' | 'Ầ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'A',
            'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
            'É' | 'È' | 'Ẻ' | 'Ẽ' | 'Ẹ' => 'E',
            'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'e',
            'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ' => 'E',
            'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            'Í' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
            'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
            'Ó' | 'Ò' | 'Ỏ' | 'Õ' | 'Ọ' => 'O',
            'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => 'o',
            'Ố' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ộ' => 'O',
            'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'o',
            'Ớ' | 'Ờ' | 'Ở' | 'Ỡ' | 'Ợ' => 'O',
            'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
            'Ú' | 'Ù' | 'Ủ' | 'Ũ' | 'Ụ' => 'U',
            'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'u',
            'Ứ' | 'Ừ' | 'Ử' | 'Ữ' | 'Ự' => 'U',
            'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
            'Ý' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'Y',
            other => other,
        }).collect()
    }

    pub fn reset() {
        UnikeyEngine::reset();
    }
}
