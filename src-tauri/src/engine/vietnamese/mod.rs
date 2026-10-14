pub mod telex;
pub mod english;
pub mod unikey_ffi;

pub use telex::VietnameseTelexEngine;
pub use unikey_ffi::UnikeyEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_telex() {
        let cases = [
            ("Tooi", "Tôi"), ("ddang", "đang"), ("gox", "gõ"), ("tieengs", "tiếng"),
            ("vieetj", "việt"), ("raats", "rất"), ("muowtj", "mượt"), ("hoas", "hóa"),
            ("hoaf", "hòa"), ("nguyeetj", "nguyệt"), ("thuyeenf", "thuyền"),
            ("chuyeenr", "chuyển"), ("tieets", "tiết"), ("bieets", "biết"),
            ("yeens", "yến"), ("yeeus", "yếu"), ("duowngf", "dường"), ("dduowngf", "đường"),
            ("queen", "quên"), ("Queen", "Quên"), ("QUEEN", "QUÊN"), ("teen", "tên"),
            ("hiep", "hiep"), ("Hiep", "Hiep"), ("HIEP", "HIEP"),
            ("hieepj", "hiệp"), ("Hieepj", "Hiệp"), ("HIEEPJ", "HIỆP"),
            ("thanhf", "thành"), ("Thanhf", "Thành"), ("THANHF", "THÀNH"),
            ("minhf", "mình"), ("canhr", "cảnh"), ("tinhx", "tĩnh"),
            ("bangr", "bảng"), ("Bangr", "Bảng"), ("BANGR", "BẢNG"),
            ("as", "á"), ("As", "Á"), ("car", "cả"), ("Car", "Cả"), ("cas", "cá"), ("bar", "bả"),
            ("caf", "cà"), ("caj", "cạ"), ("cax", "cã"), ("hats", "hát"), ("bans", "bán"),
            ("tangs", "táng"), ("cungx", "cũng"), ("dungj", "dụng"), ("dongf", "dòng"),
            ("chaua", "châu"), ("chauaa", "chaua"), ("chauaan", "chauan"),
            ("Antigravity", "Antigravity"), ("antigravity", "antigravity")
        ];
        for (input, expected) in cases {
            assert_eq!(VietnameseTelexEngine::transform(input), expected, "Failed for input '{}'", input);
        }
    }

    #[test]
    fn test_delayed_modifiers() {
        let cases = [
            ("ddongjo", "động"), ("Ddongjo", "Động"), ("DDONGJO", "ĐỘNG"),
            ("dongjo", "dộng"), ("tangfa", "tầng"), ("ddenfe", "đền"), ("denfe", "dền"),
            ("dudocjw", "được"), ("dduocjw", "được"), ("duocjw", "dược"), ("deerd", "để"), ("dangd", "đang"), ("trene", "trên"),
            ("memef", "mềm"), ("thuyeenf", "thuyền"), ("truowngf", "trường"),
            ("nguwowif", "người"), ("booj", "bộ"), ("phana", "phân"), ("nghix", "nghĩ"),
            ("cuangx", "cuãng"), ("thuwjc", "thực"), ("teej", "tệ"),
            ("vanag", "vâng"), ("Vanag", "Vâng"), ("VANAG", "VÂNG"), ("trongos", "trống"),
            ("dods", "đó"), ("Dods", "Đó"), ("DODS", "ĐÓ"), ("dads", "đá"), ("dudngs", "đúng"),
            ("lys", "lý"), ("Lys", "Lý"), ("LYS", "LÝ"), ("myx", "mỹ"), ("Myx", "Mỹ"), ("tyr", "tỷ"), ("kyf", "kỳ")
        ];
        for (input, expected) in cases {
            assert_eq!(VietnameseTelexEngine::transform(input), expected, "Failed for input '{}'", input);
        }
    }

    #[test]
    fn test_escapes_and_english() {
        let cases = [
            ("good", "good"), ("Good", "Good"), ("GOOD", "GOOD"),
            ("goodbye", "goodbye"), ("Goodbye", "Goodbye"),
            ("bye", "bye"), ("Bye", "Bye"),
            ("goood", "good"), ("Goood", "Good"), ("GOOOD", "GOOD"),
            ("tooo", "too"), ("leee", "lee"), ("caaa", "caa"), ("dddog", "ddog"),
            ("showw", "show"), ("howw", "how"), ("draww", "draw"), ("laww", "law"),
            ("ww", "w"), ("WW", "W"), ("www", "www"),
            ("tess", "tes"), ("teff", "tef"), ("terr", "ter"), ("texx", "tex"),
            ("test", "test"), ("reset", "reset"), ("shift", "shift"),
            ("facebook", "facebook"), ("google", "google"), ("youtube", "youtube"),
            ("free", "free"), ("yes", "yes"),
            ("virffresh", "vifresh"), ("VIRFFRESH", "VIFRESH"), ("Virffresh", "Vifresh"),
            ("queeen", "queen"), ("teeen", "teen"),
            ("share", "share"), ("Share", "Share"), ("SHARE", "SHARE"),
            ("care", "care"), ("Care", "Care"), ("user", "user"), ("User", "User"),
            ("fire", "fire"), ("shar", "shar"),
            ("adidas", "adidas"), ("Adidas", "Adidas"), ("ADIDAS", "ADIDAS"),
            ("status", "status"), ("canvas", "canvas"), ("asus", "asus"), ("virus", "virus"),
            ("draft", "draft"), ("craft", "craft"), ("staff", "staff"),
            ("boxx", "box"), ("taxx", "tax"), ("maxx", "max"), ("fax", "fax"),
            ("order", "order"), ("Order", "Order"), ("orrder", "order"), ("Orrder", "Order"),
            ("tauri", "tauri"), ("Tauri", "Tauri"), ("taurri", "tauri"), ("Taurri", "Tauri"), ("TAURRI", "TAURI"),
            ("ass", "as"), ("carr", "car"), ("barr", "bar"), ("hass", "has"),
            ("asus", "asus"), ("Asus", "Asus"), ("assus", "asus"), ("Assus", "Asus"),
            ("toanz", "toanz"), ("vietz", "vietz"), ("nhaz", "nhaz"), ("muaz", "muaz"),
            ("haiz", "haiz"), ("hanzi", "hanzi"), ("hanzii", "hanzii")
        ];
        for (input, expected) in cases {
            assert_eq!(VietnameseTelexEngine::transform(input), expected, "Failed for input '{}'", input);
        }
    }

    #[test]
    fn test_casing_variations() {
        let cases = [
            ("Tooi", "Tôi"), ("TOOI", "TÔI"), ("tooi", "tôi"),
            ("Vieetj", "Việt"), ("VIEETJ", "VIỆT"), ("vieetj", "việt"), ("VIeetj", "VIệt"),
            ("Dudocjw", "Được"), ("DUDOCJW", "ĐƯỢC"), ("dudocjw", "được"),
            ("Hoas", "Hóa"), ("HOAS", "HÓA"), ("hoas", "hóa"),
            ("Dd", "Đ"), ("DD", "Đ"), ("dd", "đ"),
            ("W", "Ư"), ("WW", "W"), ("Ww", "W"),
            ("Good", "Good"), ("GOOD", "GOOD"), ("good", "good"),
            ("Goodbye", "Goodbye"), ("GOODBYE", "GOODBYE"),
            ("VIRFFRESH", "VIFRESH"), ("Virffresh", "Vifresh"),
            ("Queen", "Quên"), ("QUEEN", "QUÊN"),
            ("VCKeys", "VCKeys"), ("VCKey", "VCKey"), ("VCKe", "VCKe"), ("VCK", "VCK"),
            ("iPhone", "iPhone"), ("macOS", "macOS"), ("ChatGPT", "ChatGPT"),
            ("UniKey", "UniKey"), ("AutoCAD", "AutoCAD"),
            ("CHUs", "CHÚ"), ("VIEETs", "VIẾT"), ("VIETs", "VIÉT")
        ];
        for (input, expected) in cases {
            assert_eq!(VietnameseTelexEngine::transform(input), expected, "Failed for input '{}'", input);
        }
    }

    #[test]
    fn test_unikey_cpp_engine() {
        let cases = [
            ("Tooi", "Tôi"), ("ddang", "đang"), ("gox", "gõ"), ("tieengs", "tiếng"),
            ("vieetj", "việt"), ("raats", "rất"), ("hoaf", "hòa"),
            ("thanhf", "thành"), ("Thanhf", "Thành"), ("THANHF", "THÀNH"),
            ("minhf", "mình"), ("canhr", "cảnh"), ("tinhx", "tĩnh"),
            ("nguyeetj", "nguyệt"), ("thuyeenf", "thuyền"), ("chuyeenr", "chuyển"),
            ("Antigravity", "Antigravity"), ("antigravity", "antigravity"),
            ("ddongjo", "động"), ("thuwjc", "thực"), ("dduocjw", "được"),
            ("adidas", "adidas"), ("share", "share"), ("draft", "draft"),
            ("status", "status"), ("canvas", "canvas"),
            ("low", "lơ"), ("bow", "bơ"), ("cow", "cơ"), ("now", "nơ"),
            ("mow", "mơ"), ("tow", "tơ"), ("sow", "sơ"), ("row", "rơ"),
            ("how", "hơ"), ("tuw", "tư"), ("lows", "lớ"), ("lowf", "lờ"),
            ("hair", "hải"), ("beer", "bể"), ("beef", "bề"), ("bangs", "báng"),
            ("hangs", "háng"), ("char", "chả"), ("chair", "chải"), ("chats", "chát"),
            ("theme", "thêm"), ("themes", "thếm"), ("themef", "thềm"),
            ("cana", "cân"), ("namw", "năm"),
            ("haiz", "haiz"), ("hanzii", "hanzii"), ("zalo", "zalo"), ("zen", "zen"),
            ("TOOI", "TÔI"), ("VIEETJ", "VIỆT"),
            ("SS", "SS"), ("ss", "ss"),
            ("FF", "FF"), ("ff", "ff"),
            ("RR", "RR"), ("rr", "rr"),
            ("XX", "XX"), ("xx", "xx"),
            ("JJ", "JJ"), ("jj", "jj"),
            ("CSS", "CSS"), ("css", "css"),
            ("ass", "as"), ("cass", "cas"), ("tess", "tes")
        ];
        for (input, expected) in cases {
            assert_eq!(UnikeyEngine::transform(input), expected, "UniKey C++ failed for input '{}'\n  → expected '{}', got '{}'", input, expected, UnikeyEngine::transform(input));
        }
    }

    #[test]
    fn test_modern_style_tone_placement() {
        use crate::engine::vietnamese::unikey_ffi::{UnikeyEngine, InputMethod};

        // --- Old style (modernStyle = 0): hoá, thuỷ ---
        UnikeyEngine::set_options(true, true, true, false);
        let old_hoa = UnikeyEngine::transform_with_method("hoas", InputMethod::Telex);
        let old_thuy = UnikeyEngine::transform_with_method("thuyr", InputMethod::Telex);

        // --- New style (modernStyle = 1): hóa, thủy ---
        UnikeyEngine::set_options(true, true, true, true);
        let new_hoa = UnikeyEngine::transform_with_method("hoas", InputMethod::Telex);
        let new_thuy = UnikeyEngine::transform_with_method("thuyr", InputMethod::Telex);

        // Restore default modern style
        UnikeyEngine::set_options(true, true, true, true);

        println!("Old style: hoas→{}, thuyr→{}", old_hoa, old_thuy);
        println!("New style: hoas→{}, thuyr→{}", new_hoa, new_thuy);

        assert_eq!(new_hoa, "hóa", "Modern style ON: hoas should give hóa (got {})", new_hoa);
        assert_eq!(new_thuy, "thủy", "Modern style ON: thuyr should give thủy (got {})", new_thuy);
        assert_eq!(old_hoa, "hoá", "Modern style OFF: hoas should give hoá (got {})", old_hoa);
        assert_eq!(old_thuy, "thuỷ", "Modern style OFF: thuyr should give thuỷ (got {})", old_thuy);
    }


    #[test]
    fn test_vni_input_method() {
        use crate::engine::vietnamese::unikey_ffi::InputMethod;
        let cases = [
            ("Toi6", "Tôi"), ("toan1", "toán"), ("vie6t5", "việt"), ("Vie6t5", "Việt"),
            ("VIE6T5", "VIỆT"), ("d9o6ng5", "động"), ("d9u7o7c5", "được"),
            ("thu7c5", "thực"), ("thanh2", "thành"), ("Thanh2", "Thành"),
            ("hoa1", "hóa"), ("toan10", "toan"),
            ("ly1", "lý"), ("my4", "mỹ"), ("ty3", "tỷ"),
            ("Antigravity", "Antigravity"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform_with_method(input, InputMethod::Vni),
                expected,
                "VNI failed for input '{}'",
                input
            );
        }
    }

    #[test]
    fn test_simple_telex_input_method() {
        use crate::engine::vietnamese::unikey_ffi::InputMethod;
        let cases = [
            ("tooi", "tôi"), ("vieetj", "việt"), ("thanhf", "thành"),
            ("uw", "ư"), ("ow", "ơ"), ("aw", "ă"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform_with_method(input, InputMethod::SimpleTelex),
                expected,
                "Simple Telex failed for input '{}'",
                input
            );
        }
    }

    #[test]
    fn test_viqr_input_method() {
        use crate::engine::vietnamese::unikey_ffi::InputMethod;
        let cases = [
            ("toan'", "toán"), ("vie^.t", "việt"), ("thanh`", "thành"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform_with_method(input, InputMethod::Viqr),
                expected,
                "VIQR failed for input '{}'",
                input
            );
        }
    }

    #[test]
    fn test_ww_word_handling() {
        assert_eq!(VietnameseTelexEngine::transform("W"), "Ư");
        assert_eq!(VietnameseTelexEngine::transform("WW"), "W");
        assert_eq!(VietnameseTelexEngine::transform("WWo"), "Wo");
        assert_eq!(VietnameseTelexEngine::transform("WWor"), "Wor");
        assert_eq!(VietnameseTelexEngine::transform("WWord"), "Word");
        assert_eq!(VietnameseTelexEngine::transform("ww"), "w");
        assert_eq!(VietnameseTelexEngine::transform("wwo"), "wo");
        assert_eq!(VietnameseTelexEngine::transform("wwor"), "wor");
        assert_eq!(VietnameseTelexEngine::transform("wword"), "word");
        assert_eq!(VietnameseTelexEngine::transform("wwin"), "win");
        assert_eq!(VietnameseTelexEngine::transform("wwork"), "work");
        assert_eq!(VietnameseTelexEngine::transform("wwater"), "water");
        assert_eq!(VietnameseTelexEngine::transform("wweek"), "week");
        assert_eq!(VietnameseTelexEngine::transform("wweb"), "web");
        assert_eq!(VietnameseTelexEngine::transform("wwhere"), "where");
        assert_eq!(VietnameseTelexEngine::transform("wworld"), "world");
        assert_eq!(VietnameseTelexEngine::transform("wwindows"), "windows");
        assert_eq!(VietnameseTelexEngine::transform("WWindows"), "Windows");
    }

    #[test]
    fn test_common_english_collisions() {
        let cases = [
            ("word", "word"), ("Word", "Word"), ("WORD", "WORD"),
            ("windows", "windows"), ("Windows", "Windows"),
            ("web", "web"), ("Web", "Web"),
            ("work", "work"), ("Work", "Work"),
            ("world", "world"), ("World", "World"),
            ("water", "water"), ("Water", "Water"),
            ("wifi", "wifi"), ("Wifi", "Wifi"),
            ("week", "week"), ("Week", "Week"),
            ("down", "down"), ("Down", "Down"),
            ("draw", "draw"), ("laww", "law"), ("saww", "saw"),
            ("order", "order"), ("Order", "Order"),
            ("border", "border"), ("corner", "corner"),
            ("server", "server"), ("docker", "docker"), ("rust", "rust"),
            ("cargo", "cargo"), ("react", "react"), ("vue", "vue"),
            ("status", "status"), ("canvas", "canvas"),
            ("format", "format"), ("project", "project"),
            ("clean", "clean"), ("draft", "draft"),
            ("card", "card"), ("park", "park"), ("start", "start"),
            ("bank", "bank"), ("camp", "camp"), ("point", "point"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "English collision test failed for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_z_key_preservation() {
        let cases = [
            ("haiz", "haiz"), ("Haiz", "Haiz"), ("HAIZ", "HAIZ"),
            ("hanzi", "hanzi"), ("Hanzi", "Hanzi"),
            ("hanzii", "hanzii"), ("Hanzii", "Hanzii"),
            ("pizza", "pizza"), ("Pizza", "Pizza"),
            ("zalo", "zalo"), ("Zalo", "Zalo"),
            ("zoom", "zoom"), ("zero", "zero"), ("zone", "zone"),
            ("toanz", "toanz"), ("vietz", "vietz"),
            ("nhaz", "nhaz"), ("muaz", "muaz"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "Z key test failed for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_telex_double_cancellation() {
        let cases = [
            // Double tone cancel: as (á) -> ass (as)
            ("as", "á"), ("ass", "as"),
            ("cas", "cá"), ("cass", "cas"),
            ("car", "cả"), ("carr", "car"),
            ("bar", "bả"), ("barr", "bar"),
            ("has", "há"), ("hass", "has"),
            ("tangf", "tàng"), ("tangff", "tangf"),
            ("bangr", "bảng"), ("bangrr", "bangr"),
            ("tinhx", "tĩnh"), ("tinhxx", "tinhx"),
            ("nangj", "nạng"), ("nangjj", "nangj"),
            // Double roof cancel: aa -> a, ee -> e, oo -> o
            ("chaua", "châu"), ("chauaa", "chaua"), ("chauaan", "chauan"),
            ("ddi", "đi"), ("ddid", "did"),
            ("goood", "good"), ("queeen", "queen"),
            ("teeen", "teen"), ("caaa", "caa"),
            // Double w cancel: ww -> w, showw -> show
            ("showw", "show"), ("howw", "how"), ("noww", "now"),
            ("boxx", "box"), ("taxx", "tax"), ("maxx", "max"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "Double cancellation failed for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_deferred_and_end_modifiers() {
        let cases = [
            ("theme", "thêm"), ("Theme", "Thêm"), ("THEME", "THÊM"),
            ("theem", "thêm"), ("Theem", "Thêm"),
            ("cana", "cân"), ("Cana", "Cân"),
            ("namw", "năm"), ("Namw", "Năm"),
            ("tangfa", "tầng"), ("Tangfa", "Tầng"),
            ("dudocjw", "được"), ("Dudocjw", "Được"),
            ("vanag", "vâng"), ("trongos", "trống"),
            ("dods", "đó"), ("dads", "đá"),
            ("memef", "mềm"), ("thuyeenf", "thuyền"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "Deferred modifier test failed for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_interrupted_final_consonants() {
        let cases = [
            ("cacsh", "cách"), ("Cacsh", "Cách"), ("CACSH", "CÁCH"),
            ("cacjh", "cạch"), ("Cacjh", "Cạch"), ("CACJH", "CẠCH"),
            ("tacsh", "tách"), ("tacjh", "tạch"),
            ("sacsh", "sách"), ("sacjh", "sạch"),
            ("thicsh", "thích"), ("thicjh", "thịch"),
            ("thacsh", "thách"), ("thacjh", "thạch"),
            ("licjh", "lịch"), ("licsh", "lích"),
            ("eechs", "ếch"), ("eechj", "ệch"),
            ("cansh", "cánh"), ("canjh", "cạnh"),
            ("bansh", "bánh"), ("banjh", "bạnh"),
            ("tinsh", "tính"), ("tinjh", "tịnh"),
            ("bansg", "báng"), ("banfg", "bàng"),
            ("tiensg", "tiéng"), ("tieensg", "tiếng"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "Interrupted final consonant test failed for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_double_modifier_cancellation() {
        let cases = [
            ("hiepsserver", "hiepserver"),
            ("Hiepsserver", "Hiepserver"),
            ("HIEPSSERVER", "HIEPSERVER"),
            ("hiepss", "hieps"),
            ("Hiepss", "Hieps"),
            ("vietssoft", "vietsoft"),
            ("Vietssoft", "Vietsoft"),
            ("cacss", "cacs"),
            ("Cacss", "Cacs"),
            ("lapssmart", "lapsmart"),
            ("assus", "asus"),
            ("Assus", "Asus"),
            ("orrder", "order"),
            ("Orrder", "Order"),
            ("taurri", "tauri"),
            ("Taurri", "Tauri"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "Double modifier cancellation failed for '{}'",
                input
            );
        }
    }

    #[test]
    fn test_user_reported_issues() {
        let cases = [
            // User reported issues
            ("AMAANDA", "AMANDA"),
            ("amaanda", "amanda"),
            ("Amaanda", "Amanda"),
            ("hieerm", "hiểm"),
            ("Hieerm", "Hiểm"),
            ("HIEERM", "HIỂM"),
            ("phism", "phím"),
            ("Phism", "Phím"),
            ("PHISM", "PHÍM"),
            // Tone switching on previous words
            ("confs", "cón"),
            ("Confs", "Cón"),
            ("CONFS", "CÓN"),
            ("confr", "cỏn"),
            ("confx", "cõn"),
            ("confj", "cọn"),
            ("vieetjs", "viết"),
            ("toansf", "toàn"),
            ("hocjs", "hóc"),
            // Similar tone keys before final consonant m
            ("hieesm", "hiếm"),
            ("Hieesm", "Hiếm"),
            ("hieefm", "hiềm"),
            ("hieexm", "hiễm"),
            ("hieejm", "hiệm"),
            ("dieerm", "diểm"),
            ("Dieerm", "Diểm"),
            ("carm", "cảm"),
            ("Carm", "Cảm"),
            ("tasm", "tám"),
            ("lafm", "làm"),
            ("kiesm", "kiém"),
            ("chieesm", "chiếm"),
            // Tones before other final consonants (n, t)
            ("basn", "bán"),
            ("bajn", "bạn"),
            ("toosn", "tốn"),
            ("viest", "viét"),
            // English words with similar endings that must remain English
            ("farm", "farm"),
            ("term", "term"),
            ("warm", "warm"),
            ("part", "part"),
            ("test", "test"),
            ("cost", "cost"),
            ("post", "post"),
            ("best", "best"),
            ("fast", "fast"),
            ("prism", "prism"),
            ("mechanism", "mechanism"),
            // Bracket shortcuts in Telex
            ("[", "ơ"),
            ("]", "ư"),
            ("{", "Ơ"),
            ("}", "Ư"),
            ("[[", "["),
            ("]]", "]"),
            ("{{", "{"),
            ("}}", "}"),
            ("[n", "ơn"),
            ("[i", "ơi"),
            ("[s", "ớ"),
            ("]a", "ưa"),
            ("m[", "mơ"),
            ("t]", "tư"),
            // Double tone key consonants without vowels (e.g. SS, FF, RR, XX, JJ) must NOT collapse
            ("SS", "SS"), ("ss", "ss"), ("Ss", "Ss"), ("sS", "sS"),
            ("FF", "FF"), ("ff", "ff"), ("Ff", "Ff"), ("fF", "fF"),
            ("RR", "RR"), ("rr", "rr"), ("Rr", "Rr"), ("rR", "rR"),
            ("XX", "XX"), ("xx", "xx"), ("Xx", "Xx"), ("xX", "xX"),
            ("JJ", "JJ"), ("jj", "jj"), ("Jj", "Jj"), ("jJ", "jJ"),
            ("SSS", "SSS"), ("sss", "sss"),
            ("FFF", "FFF"), ("fff", "fff"),
            ("RRR", "RRR"), ("rrr", "rrr"),
            ("XXX", "XXX"), ("xxx", "xxx"),
            ("JJJ", "JJJ"), ("jjj", "jjj"),
            ("CSS", "CSS"), ("css", "css"),
            ("RSS", "RSS"), ("rss", "rss"),
            ("SSH", "SSH"), ("ssh", "ssh"),
            ("SSL", "SSL"), ("ssl", "ssl"),
            ("SSD", "SSD"), ("ssd", "ssd"),
            ("TSS", "TSS"), ("tss", "tss"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "Test failed for input '{}'",
                input
            );
        }
    }

    #[test]
    fn test_tone_replacement_transitions() {
        let cases = [
            ("confs", "cón"),
            ("confr", "cỏn"),
            ("confx", "cõn"),
            ("confj", "cọn"),
            ("conff", "conf"),
            ("conss", "cons"),
            ("vieetjs", "viết"),
            ("toansf", "toàn"),
            ("hocjs", "hóc"),
            ("nghiengsf", "nghièng"),
            ("nhasf", "nhà"),
            ("toanfr", "toản"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "Tone transition test failed for input '{}'",
                input
            );
        }
    }

    #[test]
    fn test_comprehensive_evkey_comparison() {
        let cases = [
            // 1. Standard Telex - Vowels & Consonants
            ("tooi", "tôi"), ("Tôi", "Tôi"), ("TOOI", "TÔI"),
            ("ddang", "đang"), ("Ddang", "Đang"), ("DDANG", "ĐANG"),
            ("gox", "gõ"), ("Gox", "Gõ"), ("GOX", "GÕ"),
            ("tieengs", "tiếng"), ("Tieengs", "Tiếng"), ("TIEENGS", "TIẾNG"),
            ("vieetj", "việt"), ("Vieetj", "Việt"), ("VIEETJ", "VIỆT"),
            ("nam", "nam"), ("Nam", "Nam"), ("NAM", "NAM"),
            ("thuyeenf", "thuyền"), ("chuyeenr", "chuyển"), ("nguyeetj", "nguyệt"),
            ("phuowngj", "phượng"), ("dduowngf", "đường"), ("ruowuj", "rượu"),
            ("ngoawnf", "ngoằn"), ("ngoawrn", "ngoẳn"), ("ngoeof", "ngoèo"),
            ("thuaanf", "thuần"), ("xuaan", "xuân"), ("luaatj", "luật"),
            ("cuowif", "cười"), ("nguowif", "người"), ("chowis", "chới"), ("chowi", "chơi"), ("mowif", "mời"),
            ("guwir", "gửi"), ("chuwas", "chứa"), ("khoawns", "khoắn"),

            // 2. Bracket shortcuts (ơ, ư, Ơ, Ư) & double cancel
            ("[", "ơ"), ("]", "ư"), ("{", "Ơ"), ("}", "Ư"),
            ("[[", "["), ("]]", "]"), ("{{", "{"), ("}}", "}"),
            ("[n", "ơn"), ("[i", "ơi"), ("[s", "ớ"), ("[f", "ờ"), ("[r", "ở"), ("[x", "ỡ"), ("[j", "ợ"),
            ("]a", "ưa"), ("]i", "ưi"), ("]s", "ứ"), ("]f", "ừ"), ("]r", "ử"), ("]x", "ữ"), ("]j", "ự"),
            ("]owsc", "ước"), ("]ng", "ưng"),
            ("m[", "mơ"), ("t]", "tư"), ("c[", "cơ"),

            // 3. W key handling
            ("w", "ư"), ("W", "Ư"), ("ww", "w"), ("WW", "W"), ("www", "www"),
            ("wa", "ưa"), ("qua", "qua"), ("thuw", "thư"), ("tows", "tớ"),
            ("duowngf", "dường"), ("dduowngf", "đường"),

            // 4. Double modifier cancellation (Hủy dấu bằng gõ lặp)
            ("as", "á"), ("ass", "as"),
            ("af", "à"), ("aff", "af"),
            ("ar", "ả"), ("arr", "ar"),
            ("ax", "ã"), ("axx", "ax"),
            ("aj", "ạ"), ("ajj", "aj"),
            ("aa", "â"), ("aaa", "aa"),
            ("ee", "ê"), ("eee", "ee"),
            ("oo", "ô"), ("ooo", "oo"),
            ("dd", "đ"), ("ddd", "dd"),
            ("ow", "ơ"), ("oww", "ow"),
            ("uw", "ư"), ("uww", "uw"),
            ("aw", "ă"), ("aww", "aw"),
            ("AMAANDA", "AMANDA"), ("amaanda", "amanda"), ("Amaanda", "Amanda"),
            ("hiepsserver", "hiepserver"), ("Hiepsserver", "Hiepserver"),
            ("vietssoft", "vietsoft"), ("lapssmart", "lapsmart"),
            ("tess", "tes"), ("teff", "tef"), ("terr", "ter"), ("texx", "tex"),
            ("showw", "show"), ("boxx", "box"), ("taxx", "tax"),

            // 5. Free tone placement (Dấu tự do)
            ("nghieeng", "nghiêng"), ("nghieengs", "nghiếng"), ("nghiengs", "nghiéng"), ("nghiengr", "nghiẻng"),
            ("phism", "phím"), ("Phism", "Phím"), ("PHISM", "PHÍM"),
            ("hieerm", "hiểm"), ("Hieerm", "Hiểm"), ("HIEERM", "HIỂM"),
            ("carm", "cảm"), ("Carm", "Cảm"), ("CARM", "CẢM"),
            ("dieerm", "diểm"), ("Dieerm", "Diểm"), ("ddieerm", "điểm"), ("Ddieerm", "Điểm"),
            ("kiesm", "kiém"), ("chieesm", "chiếm"),
            ("basn", "bán"), ("bajn", "bạn"), ("toosn", "tốn"), ("viest", "viét"),
            ("cacsh", "cách"), ("Cacsh", "Cách"), ("CACSH", "CÁCH"),
            ("cansh", "cánh"), ("casnh", "cánh"),
            ("bansg", "báng"), ("basng", "báng"),
            ("thicsh", "thích"), ("tacsh", "tách"), ("licjh", "lịch"),

            // 6. English word protection (Không làm vỡ từ tiếng Anh)
            ("test", "test"), ("best", "best"), ("cost", "cost"), ("post", "post"),
            ("fast", "fast"), ("last", "last"), ("past", "past"), ("list", "list"),
            ("just", "just"), ("must", "must"), ("first", "first"), ("rest", "rest"),
            ("farm", "farm"), ("term", "term"), ("warm", "warm"), ("part", "part"),
            ("chart", "chart"), ("start", "start"), ("smart", "smart"), ("short", "short"),
            ("home", "home"), ("name", "name"), ("code", "code"), ("mode", "mode"),
            ("line", "line"), ("file", "file"), ("base", "base"), ("case", "case"),
            ("date", "date"), ("rate", "rate"), ("size", "size"), ("type", "type"),
            ("page", "page"), ("save", "save"), ("safe", "safe"),
            ("true", "true"), ("false", "false"), ("null", "null"), ("class", "class"),
            ("public", "public"), ("private", "private"), ("return", "return"),
            ("project", "project"), ("server", "server"), ("docker", "docker"),
            ("rust", "rust"), ("cargo", "cargo"), ("react", "react"), ("vue", "vue"),
            ("status", "status"), ("canvas", "canvas"), ("clean", "clean"), ("draft", "draft"),
            ("facebook", "facebook"), ("google", "google"), ("youtube", "youtube"),
            ("tauri", "tauri"), ("adidas", "adidas"), ("asus", "asus"),
            ("prism", "prism"), ("mechanism", "mechanism"),

            // 7. Case preservation & Mixed
            ("DUDOCJW", "ĐƯỢC"), ("Dudocjw", "Được"), ("dudocjw", "được"),
            ("HOAS", "HÓA"), ("Hoas", "Hóa"), ("hoas", "hóa"),
            ("CHUs", "CHÚ"), ("VIEETs", "VIẾT")
        ];

        for (input, expected) in cases {
            assert_eq!(
                VietnameseTelexEngine::transform(input),
                expected,
                "EVKey benchmark failed for input '{}'",
                input
            );
        }
    }
}
