use std::collections::HashSet;
use lazy_static::lazy_static;

const ENGLISH_DICT_RAW: &str = include_str!("english_words.txt");

lazy_static! {
    /// 35,000+ English words embedded directly in binary for instant O(1) lookup
    pub static ref ENGLISH_DICT: HashSet<&'static str> = {
        let mut s = HashSet::with_capacity(36000);
        for line in ENGLISH_DICT_RAW.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                s.insert(trimmed);
            }
        }
        // Additional brand names, common developer acronyms & short English words
        let extra = [
            "yes", "yeah", "yep", "no", "hi", "hello", "hey", "ok", "okay", "bye", "good", "goodbye",
            "tauri", "react", "vue", "svelte", "nextjs", "flutter", "electron", "vite", "cargo", "axum", "actix", "tokio", "serde",
            "vckeys", "vckey", "unikey", "evkey", "vietkey", "openkey",
            "chatgpt", "autocad", "photoshop", "illustrator", "premiere", "vscode",
            "github", "gitlab", "docker", "kubernetes", "k8s", "redis", "nginx",
            "mysql", "postgres", "mongodb", "sqlite", "graphql", "rust", "python",
            "typescript", "javascript", "golang", "java", "kotlin", "swift",
            "iphone", "ipad", "macbook", "macos", "android", "windows", "linux",
            "adidas", "nike", "puma", "reebok", "gucci", "chanel", "zara",
            "asus", "acer", "dell", "lenovo", "thinkpad", "intel", "nvidia", "gigabyte",
            "order", "orders", "ordered", "ordering", "border", "corner",
            "look", "looks", "looked", "looking", "book", "books", "took", "cook", "cooks", "hook", "hooks",
            "amanda", "term", "terms", "part", "parts", "storm", "storms", "chart", "charts", "short",
            "start", "starts", "smart", "sport", "sports", "prism", "mechanism", "organism",
            "socialism", "capitalism", "criticism", "form", "forms", "format", "farm", "farms", "warm",
            "past", "must", "rest", "guest", "west", "east", "dust", "rust", "bust", "gust", "cast", "mast",
            "lost", "nest", "pest", "quest", "blast", "boost", "frost", "ghost", "trust", "burst",
            "artist", "forest", "honest", "modest", "exist", "insist", "resist", "persist", "consist", "assist",
            "twist", "wrist", "fist", "mist", "press", "true", "false", "null", "class", "public", "private", "return",
            "per", "percent", "permission", "period", "perhaps", "perfect", "perform", "personal", "person", "perplexity"
        ];
        for w in extra {
            s.insert(w);
        }
        s
    };

    /// Initial consonant clusters that only appear at the START of foreign words
    pub static ref INITIAL_FOREIGN_CLUSTERS: Vec<&'static str> = vec![
        "bl", "br", "cl", "cr", "dr", "dw", "fl", "fr", "gl", "gr", "kl", "kr",
        "pl", "pr", "sc", "sk", "sl", "sm", "sn", "sp", "st", "str", "sw", "tw", "wr", "kn", "ps", "pn", "gn", "wh"
    ];

    /// Final / medial consonant clusters that only appear in foreign words
    pub static ref FINAL_FOREIGN_CLUSTERS: Vec<&'static str> = vec![
        "rd", "rt", "rk", "rn", "rm", "rv", "rs", "rch", "rge", "rse",
        "lt", "lk", "lp", "ld", "lf", "lm", "ls", "lth",
        "mp", "mb", "nk", "nt", "nd", "nch", "nth", "nge",
        "pt", "ct", "ft", "tch", "xt", "sh"
    ];

    /// Common foreign suffixes
    pub static ref FOREIGN_SUFFIXES: Vec<&'static str> = vec![
        "ble", "ple", "cle", "tle", "dle", "tion", "sion", "ment", "ness", "able", "ible", "ize", "ise", "ism", "ist"
    ];
}

/// Comprehensive parser to check if a token is a single valid Vietnamese syllable
/// containing Telex tone marks (s, f, r, x, j), Telex vowel/consonant modifiers (aa, ee, oo, ow, uw, aw, dd, w),
/// or deferred vowel modifiers at the end of a syllable (e.g. theme -> thêm, cana -> cân, namw -> năm).
fn is_vietnamese_telex_syllable(s: &str) -> bool {
    if s.is_empty() || s.len() > 8 {
        return false;
    }

    const TONES: [char; 5] = ['s', 'f', 'r', 'x', 'j'];
    const MODIFIERS: [&str; 8] = ["aa", "ee", "oo", "ow", "uw", "aw", "dd", "w"];

    const VN_INITIALS: [&str; 26] = [
        "ngh", "ng", "nh", "gh", "gi", "kh", "ph", "qu", "th", "tr", "dd", "ch",
        "b", "c", "d", "g", "h", "k", "l", "m", "n", "r", "s", "t", "v", "x"
    ];

    const VN_FINALS: [&str; 8] = [
        "ng", "nh", "ch", "c", "m", "n", "p", "t"
    ];

    const VN_VOWELS: [&str; 66] = [
        "ieeu", "uyee", "uoow", "yeew", "oong", "oang", "uowi", "uowu",
        "aay", "aaw", "eeu", "iee", "ieu", "oai", "oay", "oeo", "ooi", "oow", "oaw", "oee",
        "uay", "uee", "uoi", "uoo", "uow", "uya", "uye", "uyu", "uaa", "uwi", "uwa", "yee", "yeu",
        "aa", "aw", "ee", "oo", "ow", "uw", "ai", "ao", "au", "ay", "eo", "ia", "ie", "iu", "oa", "oe", "oi", "owi", "ua", "ue", "ui", "uo", "uu", "uy", "ye", "iaw",
        "a", "e", "i", "o", "u", "y", "w"
    ];

    // Case 1: Tone at end of syllable (e.g. phims, hieemr, as, car, cas, bar, has)
    let last_char = s.chars().last().unwrap();
    if TONES.contains(&last_char) {
        let rem = &s[..s.len() - 1];
        let mut r = rem;
        for ini in VN_INITIALS {
            if r.starts_with(ini) {
                r = &r[ini.len()..];
                break;
            }
        }
        for fin in VN_FINALS {
            if r.ends_with(fin) {
                r = &r[..r.len() - fin.len()];
                break;
            }
        }
        if VN_VOWELS.contains(&r) {
            return true;
        }
    }

    // Case 2: Tone before final consonant or cluster (e.g. phism -> phim, hieerm -> hieem, carm -> cam, dieerm -> dieem, basn -> ban)
    // Only applies if s is not in ENGLISH_DICT so real English words like test, cost, term remain English
    if !ENGLISH_DICT.contains(s) {
        for fin in VN_FINALS {
            if s.ends_with(fin) && s.len() > fin.len() + 1 {
                let prefix = &s[..s.len() - fin.len()];
                let tone_char = prefix.chars().last().unwrap();
                if TONES.contains(&tone_char) {
                    let mut rem = &prefix[..prefix.len() - 1];
                    for ini in VN_INITIALS {
                        if rem.starts_with(ini) {
                            rem = &rem[ini.len()..];
                            break;
                        }
                    }
                    if VN_VOWELS.contains(&rem) {
                        return true;
                    }
                }
            }
        }

        // Inside final digraph: e.g. cacsh -> cach, cansh -> canh, bansg -> bang
        if s.len() >= 4 {
            let bytes = s.as_bytes();
            let n = bytes.len();
            let c1 = bytes[n - 3] as char;
            let tone_char = bytes[n - 2] as char;
            let c2 = bytes[n - 1] as char;
            let is_digraph = (c1 == 'c' && c2 == 'h') || (c1 == 'n' && c2 == 'h') || (c1 == 'n' && c2 == 'g');
            if is_digraph && TONES.contains(&tone_char) {
                let mut rem = &s[..n - 3];
                for ini in VN_INITIALS {
                    if rem.starts_with(ini) {
                        rem = &rem[ini.len()..];
                        break;
                    }
                }
                if VN_VOWELS.contains(&rem) {
                    return true;
                }
            }
        }
    }

    // Case 3: Deferred modifier at end of syllable (e.g. "theme" -> "them" + "e", "cana" -> "can" + "a", "namw" -> "nam" + "w")
    if s.len() >= 3 {
        let end_char = s.chars().last().unwrap();
        if matches!(end_char, 'e' | 'a' | 'o' | 'w') {
            let mut b_rem = &s[..s.len() - 1];
            for ini in VN_INITIALS {
                if b_rem.starts_with(ini) {
                    b_rem = &b_rem[ini.len()..];
                    break;
                }
            }
            for fin in VN_FINALS {
                if b_rem.ends_with(fin) {
                    b_rem = &b_rem[..b_rem.len() - fin.len()];
                    break;
                }
            }
            if (end_char == 'e' && b_rem == "e")
                || (end_char == 'a' && b_rem == "a")
                || (end_char == 'o' && b_rem == "o")
                || (end_char == 'w' && matches!(b_rem, "a" | "o" | "u"))
            {
                return true;
            }
        }
    }

    // Case 4: Standard inline modifiers (aa, ee, oo, etc.)
    let has_mod = MODIFIERS.iter().any(|&m| s.contains(m));
    if has_mod {
        let mut r = s;
        for ini in VN_INITIALS {
            if r.starts_with(ini) {
                r = &r[ini.len()..];
                break;
            }
        }
        for fin in VN_FINALS {
            if r.ends_with(fin) {
                r = &r[..r.len() - fin.len()];
                break;
            }
        }
        if VN_VOWELS.contains(&r) {
            return true;
        }
    }

    false
}

/// Checks if a raw lowercase string is an English/foreign word using:
/// 1. Exact match in the 35,000+ embedded English dictionary (Layer 2)
/// 2. Initial and final foreign consonant cluster detection (Layer 1)
/// 3. Foreign suffix detection (Layer 1)
pub fn is_english_word(raw_lower: &str) -> bool {
    if raw_lower.is_empty() {
        return false;
    }

    // High-priority English words that should always remain English
    const HIGH_PRIORITY_ENGLISH: [&str; 21] = [
        "yes", "bye", "hey", "hi", "ok", "okay", "no", "react", "vue", "docker", "rust", "draft", "status", "canvas",
        "down", "town", "look", "looks", "looking", "looked", "per"
    ];
    if HIGH_PRIORITY_ENGLISH.contains(&raw_lower) {
        return true;
    }

    // Words containing "aa" that are not in the English dictionary are Vietnamese Telex inputs (e.g. "amaanda", "baanda")
    if raw_lower.contains("aa") && !ENGLISH_DICT.contains(raw_lower) {
        return false;
    }

    // Syllables that are structurally valid Vietnamese Telex inputs with tones/modifiers
    // (e.g. as -> á, car -> cả, cas -> cá, bar -> bả, has -> há, low -> lơ, hair -> hải, beer -> bể, beef -> bề,
    //  phism -> phím, hieerm -> hiểm, carm -> cảm)
    // MUST be processed as Vietnamese unless the user types duplicate tone modifiers to cancel (e.g. ass -> as, carr -> car).
    if is_vietnamese_telex_syllable(raw_lower) {
        return false;
    }

    // If starts with 'd' and contains multiple 'd's (user intentionally typing 'đ' in Vietnamese, e.g. "dads" -> "đá", "ddang" -> "đang")
    let d_count = raw_lower.chars().filter(|&c| c == 'd').count();
    if d_count >= 2 && raw_lower.starts_with('d') && raw_lower != "adidas" {
        return false;
    }

    // 1. Direct match in comprehensive English dictionary
    if ENGLISH_DICT.contains(raw_lower) {
        return true;
    }

    // Ignore active tone/hat escape keys at end of word (e.g. "tess", "teff", "showw")
    // Only applies if there is a vowel preceding the double modifier keys!
    let double_mods = ["ww", "ff", "ss", "rr", "xx", "jj"];
    for dm in double_mods {
        if raw_lower.ends_with(dm) && raw_lower.len() > dm.len() {
            let prefix = &raw_lower[..raw_lower.len() - dm.len()];
            if prefix.chars().any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')) {
                return false;
            }
        }
    }
    if raw_lower.contains("ffresh") {
        return false;
    }

    // If starts with multiple 'd's like "dd..." (user intentionally typing 'đ')
    let d_count = raw_lower.chars().filter(|&c| c == 'd').count();
    if d_count >= 2 && raw_lower.starts_with('d') && !ENGLISH_DICT.contains(raw_lower) {
        return false;
    }

    // 2. Check if word starts with a foreign initial consonant cluster (e.g. green, draft, start, project, clean)
    for ic in INITIAL_FOREIGN_CLUSTERS.iter() {
        if raw_lower.starts_with(ic) {
            return true;
        }
    }

    // 3. Check if word ends with a foreign suffix (e.g. action, development, business)
    for suf in FOREIGN_SUFFIXES.iter() {
        if raw_lower.ends_with(suf) {
            return true;
        }
    }

    // 4. Check if word contains foreign medial/final clusters (e.g. card, format, point, bank, camp)
    for fc in FINAL_FOREIGN_CLUSTERS.iter() {
        if raw_lower.contains(fc) {
            return true;
        }
    }

    // 5. Words that start with strictly non-Vietnamese initial consonants or contain 'z'
    if raw_lower.contains('z')
        || raw_lower.starts_with('f')
        || raw_lower.starts_with('j')
        || raw_lower.starts_with("wr")
        || raw_lower.starts_with("wh")
        || raw_lower.starts_with("ps")
        || raw_lower.starts_with("pn")
        || raw_lower.starts_with("gn")
        || raw_lower.starts_with("kn")
    {
        return true;
    }

    false
}
