use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::engine::chinese::pinyin_trie::tokenize_pinyin;

const PINYIN_TONE_DATA: &str = include_str!("pinyin_tone_dict.txt");

lazy_static! {
    static ref CHAR_PINYIN_MAP: HashMap<char, Vec<&'static str>> = {
        let mut map = HashMap::with_capacity(12500);
        for line in PINYIN_TONE_DATA.lines() {
            if let Some((c_str, readings)) = line.split_once('\t') {
                if let Some(c) = c_str.chars().next() {
                    let r_list: Vec<&'static str> = readings.split(',').collect();
                    map.insert(c, r_list);
                }
            }
        }
        map
    };
}

/// Loại bỏ dấu thanh điệu khỏi chuỗi pinyin để so sánh với phím gõ của người dùng
pub fn strip_tone(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'ā' | 'á' | 'ǎ' | 'à' => res.push('a'),
            'ō' | 'ó' | 'ǒ' | 'ò' => res.push('o'),
            'ē' | 'é' | 'ě' | 'è' | 'ê' | 'ế' | 'ề' => res.push('e'),
            'ī' | 'í' | 'ǐ' | 'ì' => res.push('i'),
            'ū' | 'ú' | 'ǔ' | 'ù' => res.push('u'),
            'ü' | 'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' => res.push('v'),
            'ń' | 'ň' | 'ǹ' => res.push('n'),
            'Ā' | 'Á' | 'Ǎ' | 'À' => res.push('a'),
            'Ō' | 'Ó' | 'Ǒ' | 'Ò' => res.push('o'),
            'Ē' | 'É' | 'Ě' | 'È' => res.push('e'),
            'Ī' | 'Í' | 'Ǐ' | 'Ì' => res.push('i'),
            'Ū' | 'Ú' | 'Ǔ' | 'Ù' => res.push('u'),
            'Ü' | 'Ǖ' | 'Ǘ' | 'Ǚ' | 'Ǜ' => res.push('v'),
            _ => res.push(c.to_ascii_lowercase()),
        }
    }
    res
}

/// Lấy phiên âm Pinyin có dấu cho một từ hoặc cụm từ chữ Hán,
/// kết hợp với chuỗi pinyin người dùng đã gõ để phân giải chuẩn xác chữ đa âm (polyphone).
pub fn get_pinyin_with_tone(word: &str, pinyin_input: &str) -> String {
    let chars: Vec<char> = word.chars().filter(|&c| {
        ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{3400}'..='\u{4dbf}').contains(&c)
    }).collect();

    if chars.is_empty() {
        return String::new();
    }

    let clean_input = pinyin_input.to_lowercase().replace('\'', "");
    let tokens = tokenize_pinyin(&clean_input);

    let mut result_parts = Vec::with_capacity(chars.len());

    if tokens.len() == chars.len() {
        // Khớp 1-1 từng âm tiết với từng chữ Hán
        for (i, &c) in chars.iter().enumerate() {
            let tok = &tokens[i];
            if let Some(readings) = CHAR_PINYIN_MAP.get(&c) {
                let mut matched = false;
                for &r in readings {
                    if strip_tone(r) == *tok {
                        result_parts.push(r);
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    result_parts.push(readings[0]);
                }
            } else {
                result_parts.push("");
            }
        }
    } else {
        // Fallback: Lấy phát âm mặc định chuẩn nhất cho từng chữ Hán
        for &c in &chars {
            if let Some(readings) = CHAR_PINYIN_MAP.get(&c) {
                result_parts.push(readings[0]);
            } else {
                result_parts.push("");
            }
        }
    }

    result_parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char_tones() {
        assert_eq!(get_pinyin_with_tone("你", "ni"), "nǐ");
        assert_eq!(get_pinyin_with_tone("拟", "ni"), "nǐ");
        assert_eq!(get_pinyin_with_tone("泥", "ni"), "ní");
        assert_eq!(get_pinyin_with_tone("呢", "ni"), "ní");
        assert_eq!(get_pinyin_with_tone("尼", "ni"), "ní");
    }

    #[test]
    fn test_polyphone_tones() {
        assert_eq!(get_pinyin_with_tone("长", "chang"), "cháng");
        assert_eq!(get_pinyin_with_tone("长", "zhang"), "zhǎng");
        assert_eq!(get_pinyin_with_tone("重", "chong"), "chóng");
        assert_eq!(get_pinyin_with_tone("重", "zhong"), "zhòng");
    }

    #[test]
    fn test_multi_char_phrase_tones() {
        assert_eq!(get_pinyin_with_tone("你好", "nihao"), "nǐ hǎo");
        assert_eq!(get_pinyin_with_tone("仓库部", "cangkubu"), "cāng kù bù");
        assert_eq!(get_pinyin_with_tone("中国", "zhongguo"), "zhōng guó");
    }

    #[test]
    fn test_jianpin_tones() {
        assert_eq!(get_pinyin_with_tone("公司", "gs"), "gōng sī");
        assert_eq!(get_pinyin_with_tone("中国", "zg"), "zhōng guó");
        assert_eq!(get_pinyin_with_tone("北京", "bj"), "běi jīng");
    }
}
