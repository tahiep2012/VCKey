pub mod pinyin_trie;
pub mod s2t;
pub mod emoji;
pub mod candidate_window;
pub mod user_dict;
pub mod v_mode;
pub mod pinyin_tone;

pub use pinyin_trie::{ChineseEngine, tokenize_pinyin, Candidate};
pub use s2t::convert_s2t;
pub use emoji::get_emoji_suggestions;
pub use candidate_window::{show_candidate_popup, hide_candidate_popup, start_candidate_window_thread};
pub use user_dict::UserDict;
pub use v_mode::process_v_mode;
pub use pinyin_tone::get_pinyin_with_tone;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let tokens = tokenize_pinyin("nihaoshijie");
        assert_eq!(tokens, vec!["ni", "hao", "shi", "jie"]);
    }

    #[test]
    fn test_sentence_composition() {
        let engine = ChineseEngine::new();
        let res_jishubu = engine.lookup("jishubu");
        assert!(!res_jishubu.is_empty());
        assert_eq!(res_jishubu[0].text, "技术部");

        let res_jishu = engine.lookup("jishu");
        assert!(!res_jishu.is_empty());
        assert_eq!(res_jishu[0].text, "技术");

        let res = engine.lookup("jintiantianqihenhao");
        assert!(!res.is_empty());
        assert_eq!(res[0].text, "今天天气很好");

        let res2 = engine.lookup("womenquchifanba");
        assert!(!res2.is_empty());
        assert_eq!(res2[0].text, "我们去吃饭吧");
    }

    #[test]
    fn test_s2t_conversion() {
        assert_eq!(convert_s2t("简体输入法"), "簡體輸入法");
        assert_eq!(convert_s2t("今天天气很好"), "今天天氣很好");
    }

    #[test]
    fn test_emoji_suggestions() {
        let emojis = get_emoji_suggestions("kaixin");
        assert_eq!(emojis, vec!["😊", "😄", "😃"]);
    }

    #[test]
    fn test_user_dict_priority() {
        let engine = ChineseEngine::new();
        let mut ud = UserDict {
            entries: std::collections::HashMap::new(),
            path: std::path::PathBuf::from("test_only.json"),
        };
        // Thêm cụm từ do người dùng ghép: cangkubu → 仓库部
        ud.add_phrase("cangkubu", "仓库部");

        // Kiểm tra exact match: User dict trước, sau đó vẫn còn gợi ý hệ thống (仓库不)
        let results = engine.lookup_with_user_dict("cangkubu", &ud);
        assert!(!results.is_empty());
        assert_eq!(results[0].text, "仓库部");
        assert_eq!(results[0].weight, 9999);
        assert!(results.iter().any(|c| c.text == "仓库不"), "System suggestion 仓库不 must also be in candidate list");

        // Kiểm tra ghép động: cangkubuwo = 仓库部 + 我 (KHÔNG phải 仓库不我)
        let results2 = engine.lookup_with_user_dict("cangkubuwo", &ud);
        assert!(!results2.is_empty(), "cangkubuwo must return at least one candidate");
        // Ứng viên đầu tiên phải là 仓库部我 (user dict 仓库部 + system 我)
        let first = &results2[0].text;
        assert_eq!(first, "仓库部我",
            "Expected 仓库部我 but got {}. User dict must be used during sub-phrase composition.", first);
        assert!(results2.iter().any(|c| c.text == "仓库不我"), "System suggestion 仓库不我 must also be present as alternative");
    }

    #[test]
    fn test_jianpin_initials() {
        let engine = ChineseEngine::new();

        // 1. System jianpin: "gs" should suggest "公司"
        let res_gs = engine.lookup("gs");
        assert!(!res_gs.is_empty(), "gs should return candidates");
        assert!(res_gs.iter().any(|c| c.text == "公司"), "gs must contain 公司");

        // 2. System jianpin: "zg" should suggest "中国"
        let res_zg = engine.lookup("zg");
        assert!(!res_zg.is_empty(), "zg should return candidates");
        assert!(res_zg.iter().any(|c| c.text == "中国"), "zg must contain 中国");

        // 3. System jianpin: "bj" should suggest "北京"
        let res_bj = engine.lookup("bj");
        assert!(!res_bj.is_empty(), "bj should return candidates");
        assert!(res_bj.iter().any(|c| c.text == "北京"), "bj must contain 北京");

        // 4. System jianpin: "nh" should suggest "你好"
        let res_nh = engine.lookup("nh");
        assert!(!res_nh.is_empty(), "nh should return candidates");
        assert!(res_nh.iter().any(|c| c.text == "你好"), "nh must contain 你好");

        // 5. System jianpin: "rmb" should suggest "人民币"
        let res_rmb = engine.lookup("rmb");
        assert!(!res_rmb.is_empty(), "rmb should return candidates");
        assert!(res_rmb.iter().any(|c| c.text == "人民币"), "rmb must contain 人民币");

        // 6. User dict jianpin: "ckb" for "cangkubu" -> "仓库部"
        let mut ud = UserDict {
            entries: std::collections::HashMap::new(),
            path: std::path::PathBuf::from("test_only_jp.json"),
        };
        ud.add_phrase("cangkubu", "仓库部");
        let res_ckb = engine.lookup_with_user_dict("ckb", &ud);
        assert!(!res_ckb.is_empty(), "ckb must return candidates with user dict");
        assert!(res_ckb.iter().any(|c| c.text == "仓库部"), "ckb must contain 仓库部 from user dict");
    }

    #[test]
    fn test_v_mode() {
        let engine = ChineseEngine::new();

        // 1. Number to Chinese lower & upper (v1234)
        let res_num = engine.lookup("v1234");
        assert!(!res_num.is_empty(), "v1234 must return candidates");
        assert_eq!(res_num[0].text, "一千二百三十四");
        assert_eq!(res_num[1].text, "壹仟贰佰叁拾肆");
        assert_eq!(res_num[2].text, "壹仟贰佰叁拾肆元整");

        // 2. Decimal number (v1234.56)
        let res_dec = engine.lookup("v1234.56");
        assert!(!res_dec.is_empty(), "v1234.56 must return candidates");
        assert_eq!(res_dec[0].text, "一千二百三十四点五六");
        assert_eq!(res_dec[1].text, "壹仟贰佰叁拾肆元伍角陆分");

        // 3. Date conversion (v2026.08.26)
        let res_date = engine.lookup("v2026.08.26");
        assert!(!res_date.is_empty(), "v2026.08.26 must return candidates");
        assert_eq!(res_date[0].text, "2026年08月26日");
        assert_eq!(res_date[1].text, "二〇二六年八月二十六日");
        assert_eq!(res_date[2].text, "2026-08-26");

        // 4. Date conversion 8-digit (v20260826)
        let res_date2 = engine.lookup("v20260826");
        assert!(!res_date2.is_empty(), "v20260826 must return candidates");
        assert_eq!(res_date2[0].text, "2026年08月26日");
        assert_eq!(res_date2[1].text, "二〇二六年八月二十六日");

        // 5. Math calculation (v100*25)
        let res_math = engine.lookup("v100*25");
        assert!(!res_math.is_empty(), "v100*25 must return candidates");
        assert_eq!(res_math[0].text, "2500");
        assert_eq!(res_math[1].text, "100*25=2500");
        assert_eq!(res_math[2].text, "二千五百");
        assert_eq!(res_math[3].text, "贰仟伍佰");

        // 6. Current date/time shortcut (vrq)
        let res_rq = engine.lookup("vrq");
        assert!(!res_rq.is_empty(), "vrq must return candidates");
        assert!(res_rq[0].text.contains("年"));
        assert!(res_rq[0].text.contains("月"));
        assert!(res_rq[0].text.contains("日"));
    }
}
