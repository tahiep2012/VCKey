use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EMOJI_MAP: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("kaixin", vec!["😊", "😄", "😃"]);
        m.insert("xiao", vec!["😄", "😆", "😂"]);
        m.insert("ku", vec!["😭", "😢"]);
        m.insert("aixin", vec!["❤️", "💖", "💕"]);
        m.insert("zan", vec!["👍", "👏"]);
        m.insert("dianhua", vec!["📞", "📱"]);
        m.insert("shouji", vec!["📱", "📲"]);
        m.insert("diannao", vec!["💻", "🖥️"]);
        m.insert("qian", vec!["💰", "💵"]);
        m.insert("huoche", vec!["🚆", "🚂"]);
        m.insert("feiji", vec!["✈️", "🛫"]);
        m.insert("qiche", vec!["🚗", "🏎️"]);
        m.insert("taiyang", vec!["☀️", "🌞"]);
        m.insert("yueliang", vec!["🌙", "🌕"]);
        m.insert("xingxing", vec!["⭐", "✨"]);
        m
    };
}

pub fn get_emoji_suggestions(pinyin: &str) -> Vec<String> {
    if let Some(list) = EMOJI_MAP.get(pinyin) {
        list.iter().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}
