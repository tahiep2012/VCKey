use std::collections::{HashMap, HashSet};
use lazy_static::lazy_static;

const DICT_DATA: &str = include_str!("dict.txt");

lazy_static! {
    pub static ref SYLLABLES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        let list = [
            "a", "ai", "an", "ang", "ao", "ba", "bai", "ban", "bang", "bao", "bei", "ben", "beng", "bi",
            "bian", "biao", "bie", "bin", "bing", "bo", "bu", "ca", "cai", "can", "cang", "cao", "ce",
            "cen", "ceng", "cha", "chai", "chan", "chang", "chao", "che", "chen", "cheng", "chi", "chong",
            "chou", "chu", "chua", "chuai", "chuan", "chuang", "chui", "chun", "chuo", "ci", "cong", "cou",
            "cu", "cuan", "cui", "cun", "cuo", "da", "dai", "dan", "dang", "dao", "de", "dei", "den",
            "deng", "di", "dia", "dian", "diao", "die", "ding", "diu", "dong", "dou", "du", "duan", "dui",
            "dun", "duo", "e", "ei", "en", "eng", "er", "fa", "fan", "fang", "fei", "fen", "feng", "fo",
            "fou", "fu", "ga", "gai", "gan", "gang", "gao", "ge", "gei", "gen", "geng", "gong", "gou",
            "gu", "gua", "guai", "guan", "guang", "gui", "gun", "guo", "ha", "hai", "han", "hang", "hao",
            "he", "hei", "hen", "heng", "hong", "hou", "hu", "hua", "huai", "huan", "huang", "hui", "hun",
            "huo", "ji", "jia", "jian", "jiang", "jiao", "jie", "jin", "jing", "jiong", "jiu", "ju", "juan",
            "jue", "jun", "ka", "kai", "kan", "kang", "kao", "ke", "kei", "ken", "keng", "kong", "kou",
            "ku", "kua", "kuai", "kuan", "kuang", "kui", "kun", "kuo", "la", "lai", "lan", "lang", "lao",
            "le", "lei", "leng", "li", "lia", "lian", "liang", "liao", "lie", "lin", "ling", "liu", "lo",
            "long", "lou", "lu", "luan", "lue", "lun", "luo", "lv", "lve", "ma", "mai", "man", "mang", "mao",
            "me", "mei", "men", "meng", "mi", "mian", "miao", "mie", "min", "ming", "miu", "mo", "mou",
            "mu", "na", "nai", "nan", "nang", "nao", "ne", "nei", "nen", "neng", "ni", "nian", "niang",
            "niao", "nie", "nin", "ning", "niu", "nong", "nou", "nu", "nuan", "nue", "nun", "nuo", "nv",
            "nve", "o", "ou", "pa", "pai", "pan", "pang", "pao", "pei", "pen", "peng", "pi", "pian", "piao",
            "pie", "pin", "ping", "po", "pou", "pu", "qi", "qia", "qian", "qiang", "qiao", "qie", "qin",
            "qing", "qiong", "qiu", "qu", "quan", "que", "qun", "ran", "rang", "rao", "re", "ren", "reng",
            "ri", "rong", "rou", "ru", "rua", "ruan", "rui", "run", "ruo", "sa", "sai", "san", "sang",
            "sao", "se", "sen", "seng", "sha", "shai", "shan", "shang", "shao", "she", "shei", "shen",
            "sheng", "shi", "shou", "shu", "shua", "shuai", "shuan", "shuang", "shui", "shun", "shuo",
            "si", "song", "sou", "su", "suan", "sui", "sun", "suo", "ta", "tai", "tan", "tang", "tao",
            "te", "tei", "teng", "ti", "tian", "tiao", "tie", "ting", "tong", "tou", "tu", "tuan", "tui",
            "tun", "tuo", "wa", "wai", "wan", "wang", "wei", "wen", "weng", "wo", "wu", "xi", "xia",
            "xian", "xiang", "xiao", "xie", "xin", "xing", "xiong", "xiu", "xu", "xuan", "xue", "xun",
            "ya", "yan", "yang", "yao", "ye", "yi", "yin", "ying", "yo", "yong", "you", "yu", "yuan",
            "yue", "yun", "za", "zai", "zan", "zang", "zao", "ze", "zei", "zen", "zeng", "zha", "zhai",
            "zhan", "zhang", "zhao", "zhe", "zhei", "zhen", "zheng", "zhi", "zhong", "zhou", "zhu",
            "zhua", "zhuai", "zhuan", "zhuang", "zhui", "zhun", "zhuo", "zi", "zong", "zou", "zu",
            "zuan", "zui", "zun", "zuo"
        ];
        for item in list {
            s.insert(item);
        }
        s
    };
}

pub fn tokenize_pinyin(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let parts: Vec<&str> = lower.split('\'').collect();
    let mut res = Vec::new();

    for part in parts {
        let chars: Vec<char> = part.chars().collect();
        let n = chars.len();
        let mut pos = 0;
        while pos < n {
            let mut matched = false;
            let max_l = 6.min(n - pos);
            for l in (1..=max_l).rev() {
                let sub: String = chars[pos..pos + l].iter().collect();
                if SYLLABLES.contains(sub.as_str()) {
                    res.push(sub);
                    pos += l;
                    matched = true;
                    break;
                }
            }
            if !matched {
                res.push(chars[pos].to_string());
                pos += 1;
            }
        }
    }

    res
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub text: String,
    pub pinyin: String,
    pub weight: usize,
}

pub struct ChineseEngine {
    dict: HashMap<String, Vec<Candidate>>,
    jianpin_dict: HashMap<String, Vec<Candidate>>,
}

impl ChineseEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            dict: HashMap::with_capacity(460000),
            jianpin_dict: HashMap::with_capacity(100000),
        };
        engine.init_dict();
        engine
    }

    fn init_dict(&mut self) {
        for line in DICT_DATA.lines() {
            if let Some((py, words)) = line.split_once(':') {
                let py = py.trim();
                let mut list = Vec::new();
                let mut weight = 1000;
                for w in words.split(',') {
                    let w = w.trim();
                    if !w.is_empty() {
                        list.push(Candidate {
                            text: w.to_string(),
                            pinyin: py.to_string(),
                            weight,
                        });
                        weight = weight.saturating_sub(1).max(1);
                    }
                }
                if !list.is_empty() {
                    self.dict.insert(py.to_string(), list.clone());

                    // Index Jianpin (Initials abbreviation) for multi-syllable phrases (2+ syllables)
                    let tokens = tokenize_pinyin(py);
                    if tokens.len() >= 2 {
                        let primary_jp: String = tokens.iter().filter_map(|t| t.chars().next()).collect();
                        if !primary_jp.is_empty() {
                            let entry = self.jianpin_dict.entry(primary_jp.clone()).or_insert_with(Vec::new);
                            if entry.len() < 300 {
                                for cand in list.iter() {
                                    if !entry.iter().any(|c| c.text == cand.text) {
                                        entry.push(cand.clone());
                                        if entry.len() >= 300 {
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // Also support multi-letter initial variants for zh, ch, sh (e.g. "zhg" for zhongguo)
                        let has_compound_initial = tokens.iter().any(|t| t.starts_with("zh") || t.starts_with("ch") || t.starts_with("sh"));
                        if has_compound_initial {
                            let mut compound_jp = String::new();
                            for t in &tokens {
                                if t.starts_with("zh") {
                                    compound_jp.push_str("zh");
                                } else if t.starts_with("ch") {
                                    compound_jp.push_str("ch");
                                } else if t.starts_with("sh") {
                                    compound_jp.push_str("sh");
                                } else if let Some(c) = t.chars().next() {
                                    compound_jp.push(c);
                                }
                            }
                            if compound_jp != primary_jp && !compound_jp.is_empty() {
                                let entry = self.jianpin_dict.entry(compound_jp).or_insert_with(Vec::new);
                                if entry.len() < 300 {
                                    for cand in list.iter() {
                                        if !entry.iter().any(|c| c.text == cand.text) {
                                            entry.push(cand.clone());
                                            if entry.len() >= 300 {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn lookup(&self, raw: &str) -> Vec<Candidate> {
        if raw.starts_with('v') || raw.starts_with('V') {
            let v_res = crate::engine::chinese::v_mode::process_v_mode(raw);
            if !v_res.is_empty() {
                return v_res;
            }
        }

        let clean = raw.to_lowercase().replace('\'', "");
        if clean.is_empty() {
            return Vec::new();
        }

        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();

        // 1. Direct exact match in dictionary (e.g. "gongsi" -> "公司", "jishu" -> "技术", ...)
        if let Some(list) = self.dict.get(&clean) {
            for c in list {
                if seen.insert(c.text.clone()) {
                    results.push(c.clone());
                }
            }
        }

        // 2. Jianpin (Initials abbreviation) match (e.g. "gs" -> "公司", "zg" -> "中国", "bj" -> "北京", "nh" -> "你好")
        if let Some(jp_list) = self.jianpin_dict.get(&clean) {
            for c in jp_list {
                if seen.insert(c.text.clone()) {
                    results.push(c.clone());
                }
            }
        }

        // 2. Dynamic sentence / multi-token composition if more than 1 token
        let tokens = tokenize_pinyin(&clean);
        if tokens.len() > 1 {
            let mut composed = String::new();
            let mut idx = 0;
            let n = tokens.len();
            let mut all_matched = true;

            while idx < n {
                let mut matched = false;
                let max_len = 4.min(n - idx);
                for l in (1..=max_len).rev() {
                    let phrase_pinyin: String = tokens[idx..idx + l].concat();
                    if let Some(list) = self.dict.get(&phrase_pinyin) {
                        if !list.is_empty() {
                            composed.push_str(&list[0].text);
                            idx += l;
                            matched = true;
                            break;
                        }
                    }
                }
                if !matched {
                    all_matched = false;
                    composed.push_str(&tokens[idx]);
                    idx += 1;
                }
            }

            if all_matched && !composed.is_empty() {
                // If composed text is already present, don't duplicate; otherwise insert
                if !results.iter().any(|c| c.text == composed) {
                    if results.is_empty() {
                        results.push(Candidate {
                            text: composed,
                            pinyin: clean.clone(),
                            weight: 2000,
                        });
                    } else {
                        // Put composed sentence after top words
                        results.push(Candidate {
                            text: composed,
                            pinyin: clean.clone(),
                            weight: 500,
                        });
                    }
                }
            }
        }

        results
    }

    /// Tra cứu kết hợp: user dict được ưu tiên tuyệt đối ở MỌI cấp ghép động.
    ///
    /// Ví dụ: user dict có "cangkubu"→"仓库部".
    /// Khi người dùng gõ "cangkubuwo", hàm sẽ:
    ///  1. Tra user dict exact → không có "cangkubuwo"
    ///  2. Ghép động, ở bước idx=0 sẽ thử span 6→5→4→3→2→1 token:
    ///     - "cangkubu" (3 tokens) → tìm thấy trong user dict → ghép "仓库部"
    ///     - "wo" → tra hệ thống → "我"
    ///     → Kết quả: "仓库部我" ✓ (thay vì "仓库不我")
    pub fn lookup_with_user_dict(
        &self,
        raw: &str,
        user_dict: &crate::engine::chinese::UserDict,
    ) -> Vec<Candidate> {
        if raw.starts_with('v') || raw.starts_with('V') {
            let v_res = crate::engine::chinese::v_mode::process_v_mode(raw);
            if !v_res.is_empty() {
                return v_res;
            }
        }

        let clean = raw.to_lowercase().replace('\'', "");
        if clean.is_empty() {
            return Vec::new();
        }

        let mut seen = std::collections::HashSet::new();
        let mut results: Vec<Candidate> = Vec::new();

        // 1. User dict EXACT match on full string (highest priority, weight 9999)
        for c in user_dict.get_candidates(&clean) {
            if seen.insert(c.text.clone()) {
                results.push(c);
            }
        }

        // 2. System dict EXACT match on full string
        if let Some(list) = self.dict.get(&clean) {
            for c in list {
                if seen.insert(c.text.clone()) {
                    results.push(c.clone());
                }
            }
        }

        // 3. User dict JIANPIN match (e.g. user typed initials matching a user phrase)
        for (py, words) in &user_dict.entries {
            let tokens = tokenize_pinyin(py);
            if tokens.len() >= 2 {
                let jp: String = tokens.iter().filter_map(|t| t.chars().next()).collect();
                if jp == clean {
                    for word in words {
                        if seen.insert(word.clone()) {
                            results.push(Candidate {
                                text: word.clone(),
                                pinyin: py.clone(),
                                weight: 9500,
                            });
                        }
                    }
                }
            }
        }

        // 3. Dynamic composition — user dict checked AT EACH STEP before system dict
        let tokens = tokenize_pinyin(&clean);
        if tokens.len() > 1 {
            let mut composed = String::new();
            let mut idx = 0;
            let n = tokens.len();
            let mut all_matched = true;

            while idx < n {
                let mut matched = false;
                // Allow up to 8 tokens so long user phrases (e.g. 4-char) are found
                let max_len = 8.min(n - idx);

                for l in (1..=max_len).rev() {
                    let phrase_pinyin: String = tokens[idx..idx + l].concat();

                    // a) Check user dict first — highest priority
                    let user_cands = user_dict.get_candidates(&phrase_pinyin);
                    if !user_cands.is_empty() {
                        composed.push_str(&user_cands[0].text);
                        idx += l;
                        matched = true;
                        break;
                    }

                    // b) Check system dict (only for spans up to 4 tokens, matching original logic)
                    if l <= 4 {
                        if let Some(list) = self.dict.get(&phrase_pinyin) {
                            if !list.is_empty() {
                                composed.push_str(&list[0].text);
                                idx += l;
                                matched = true;
                                break;
                            }
                        }
                    }
                }

                if !matched {
                    all_matched = false;
                    composed.push_str(&tokens[idx]);
                    idx += 1;
                }
            }

            if all_matched && !composed.is_empty() {
                if seen.insert(composed.clone()) {
                    let weight = if results.is_empty() { 2000 } else { 500 };
                    results.push(Candidate {
                        text: composed,
                        pinyin: clean.clone(),
                        weight,
                    });
                }
            }
        }

        // 4. System dictionary dynamic composition fallback (e.g. "仓库不" from cangku + bu)
        // Ensure system suggestions are always available after user-learned words
        for sys_cand in self.lookup(raw) {
            if seen.insert(sys_cand.text.clone()) {
                results.push(sys_cand);
            }
        }

        results
    }
}
