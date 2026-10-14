use crate::engine::chinese::Candidate;

#[cfg(windows)]
fn get_current_system_time() -> (u32, u32, u32, u32, u32, u32) {
    #[repr(C)]
    struct SYSTEMTIME {
        w_year: u16,
        w_month: u16,
        w_day_of_week: u16,
        w_day: u16,
        w_hour: u16,
        w_minute: u16,
        w_second: u16,
        w_milliseconds: u16,
    }
    extern "system" {
        fn GetLocalTime(lp_system_time: *mut SYSTEMTIME);
    }
    unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);
        (
            st.w_year as u32,
            st.w_month as u32,
            st.w_day as u32,
            st.w_hour as u32,
            st.w_minute as u32,
            st.w_second as u32,
        )
    }
}

#[cfg(not(windows))]
fn get_current_system_time() -> (u32, u32, u32, u32, u32, u32) {
    (2026, 8, 26, 14, 45, 0)
}

/// Xử lý chế độ V-Mode: Bắt đầu bằng 'v' hoặc 'V'
pub fn process_v_mode(raw: &str) -> Vec<Candidate> {
    let raw = raw.trim();
    if !raw.starts_with('v') && !raw.starts_with('V') {
        return Vec::new();
    }

    let query = &raw[1..];
    if query.is_empty() {
        return vec![
            Candidate {
                text: "【V模式】输入数字/日期/算式 (如: v1234, v2026.08.26, v1+2, vrq, vsj)".to_string(),
                pinyin: raw.to_string(),
                weight: 9999,
            }
        ];
    }

    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // 1. Phím tắt ngày giờ hiện tại (vrq / vdate, vsj / vtime, vnf, vnl)
    if query_lower == "rq" || query_lower == "date" {
        let (year, month, day, _, _, _) = get_current_system_time();

        results.push(Candidate {
            text: format!("{:04}年{:02}月{:02}日", year, month, day),
            pinyin: raw.to_string(),
            weight: 9999,
        });
        results.push(Candidate {
            text: format!("{}年{}月{}日", format_year_chinese(year), number_to_chinese_lower(month as u64), number_to_chinese_lower(day as u64)),
            pinyin: raw.to_string(),
            weight: 9998,
        });
        results.push(Candidate {
            text: format!("{:04}-{:02}-{:02}", year, month, day),
            pinyin: raw.to_string(),
            weight: 9997,
        });
        results.push(Candidate {
            text: format!("{:04}/{:02}/{:02}", year, month, day),
            pinyin: raw.to_string(),
            weight: 9996,
        });
        return results;
    }

    if query_lower == "sj" || query_lower == "time" {
        let (_, _, _, hour, min, sec) = get_current_system_time();
        results.push(Candidate {
            text: format!("{:02}:{:02}:{:02}", hour, min, sec),
            pinyin: raw.to_string(),
            weight: 9999,
        });
        results.push(Candidate {
            text: format!("{:02}点{:02}分", hour, min),
            pinyin: raw.to_string(),
            weight: 9998,
        });
        results.push(Candidate {
            text: format!("{:02}时{:02}分{:02}秒", hour, min, sec),
            pinyin: raw.to_string(),
            weight: 9997,
        });
        return results;
    }

    // 2. Chuyển đổi ngày tháng: v2026.08.26 / v2026-08-26 / v2026/08/26 / v20260826
    if let Some(date_cands) = try_parse_date(query, raw) {
        if !date_cands.is_empty() {
            results.extend(date_cands);
            return results;
        }
    }

    // 3. Tính toán biểu thức toán học (e.g. v1+2, v100*25, v500-120, v100/4)
    if query.contains('+') || query.contains('-') || query.contains('*') || query.contains('/') {
        if let Some(calc_res) = eval_math_expression(query) {
            results.push(Candidate {
                text: format!("{}", calc_res),
                pinyin: raw.to_string(),
                weight: 9999,
            });
            results.push(Candidate {
                text: format!("{}={}", query, calc_res),
                pinyin: raw.to_string(),
                weight: 9998,
            });
            if calc_res >= 0.0 && calc_res <= 1_000_000_000_000.0 && calc_res.fract() == 0.0 {
                let int_val = calc_res as u64;
                results.push(Candidate {
                    text: number_to_chinese_lower(int_val),
                    pinyin: raw.to_string(),
                    weight: 9997,
                });
                results.push(Candidate {
                    text: number_to_chinese_upper(int_val),
                    pinyin: raw.to_string(),
                    weight: 9996,
                });
            }
            return results;
        }
    }

    // 4. Chuyển đổi số sang chữ Hán: v1234, v1234.56
    if let Ok(num_val) = query.parse::<f64>() {
        let is_int = num_val.fract() == 0.0 && !query.contains('.');
        if is_int && num_val >= 0.0 && num_val <= 1_000_000_000_000_000.0 {
            let n = num_val as u64;
            let lower_str = number_to_chinese_lower(n);
            let upper_str = number_to_chinese_upper(n);
            let rmb_str = format!("{}元整", upper_str);

            results.push(Candidate {
                text: lower_str,
                pinyin: raw.to_string(),
                weight: 9999,
            });
            results.push(Candidate {
                text: upper_str,
                pinyin: raw.to_string(),
                weight: 9998,
            });
            results.push(Candidate {
                text: rmb_str,
                pinyin: raw.to_string(),
                weight: 9997,
            });

            // Gợi ý số La Mã nếu n <= 3999
            if n >= 1 && n <= 3999 {
                results.push(Candidate {
                    text: to_roman_numerals(n as u32),
                    pinyin: raw.to_string(),
                    weight: 9996,
                });
            }

            // Gợi ý số tròn khoanh nếu n <= 20
            if n >= 1 && n <= 20 {
                let circled = ['①','②','③','④','⑤','⑥','⑦','⑧','⑨','⑩','⑪','⑫','⑬','⑭','⑮','⑯','⑰','⑱','⑲','⑳'];
                results.push(Candidate {
                    text: circled[(n - 1) as usize].to_string(),
                    pinyin: raw.to_string(),
                    weight: 9995,
                });
            }
        } else if num_val >= 0.0 && num_val <= 1_000_000_000_000.0 {
            // Số thập phân (e.g. 1234.56)
            let parts: Vec<&str> = query.split('.').collect();
            if parts.len() == 2 {
                let int_part: u64 = parts[0].parse().unwrap_or(0);
                let dec_part = parts[1];
                let lower_int = number_to_chinese_lower(int_part);
                let upper_int = number_to_chinese_upper(int_part);

                let lower_dec: String = dec_part.chars().map(digit_to_chinese_char_lower).collect();
                let lower_full = format!("{}点{}", lower_int, lower_dec);

                // Tiền tệ RMB với phần thập phân
                let mut rmb_full = format!("{}元", upper_int);
                let dec_chars: Vec<char> = dec_part.chars().collect();
                if !dec_chars.is_empty() {
                    let jiao = dec_chars[0].to_digit(10).unwrap_or(0);
                    if jiao > 0 {
                        rmb_full.push_str(&format!("{}角", digit_to_chinese_char_upper(dec_chars[0])));
                    } else if dec_chars.len() > 1 && dec_chars[1] != '0' {
                        rmb_full.push_str("零");
                    }
                    if dec_chars.len() > 1 {
                        let fen = dec_chars[1].to_digit(10).unwrap_or(0);
                        if fen > 0 {
                            rmb_full.push_str(&format!("{}分", digit_to_chinese_char_upper(dec_chars[1])));
                        }
                    }
                }

                results.push(Candidate {
                    text: lower_full,
                    pinyin: raw.to_string(),
                    weight: 9999,
                });
                results.push(Candidate {
                    text: rmb_full,
                    pinyin: raw.to_string(),
                    weight: 9998,
                });
                results.push(Candidate {
                    text: format!("{}点{}", upper_int, dec_part.chars().map(digit_to_chinese_char_upper).collect::<String>()),
                    pinyin: raw.to_string(),
                    weight: 9997,
                });
            }
        }
    }

    results
}

fn try_parse_date(query: &str, raw: &str) -> Option<Vec<Candidate>> {
    let clean = query.replace(['-', '/', ' '], ".");
    let parts: Vec<&str> = clean.split('.').filter(|s| !s.is_empty()).collect();

    // Trường hợp 1: YYYY.MM.DD (hoặc YYYY.M.D)
    if parts.len() == 3 {
        let year: u32 = parts[0].parse().ok()?;
        let month: u32 = parts[1].parse().ok()?;
        let day: u32 = parts[2].parse().ok()?;

        if year >= 1000 && year <= 9999 && month >= 1 && month <= 12 && day >= 1 && day <= 31 {
            let mut cands = Vec::new();
            cands.push(Candidate {
                text: format!("{:04}年{:02}月{:02}日", year, month, day),
                pinyin: raw.to_string(),
                weight: 9999,
            });
            cands.push(Candidate {
                text: format!("{}年{}月{}日", format_year_chinese(year), number_to_chinese_lower(month as u64), number_to_chinese_lower(day as u64)),
                pinyin: raw.to_string(),
                weight: 9998,
            });
            cands.push(Candidate {
                text: format!("{:04}-{:02}-{:02}", year, month, day),
                pinyin: raw.to_string(),
                weight: 9997,
            });
            cands.push(Candidate {
                text: format!("{}年{}月{}日", year, month, day),
                pinyin: raw.to_string(),
                weight: 9996,
            });
            cands.push(Candidate {
                text: format!("{}年{}月{}日", format_year_upper(year), number_to_chinese_upper(month as u64), number_to_chinese_upper(day as u64)),
                pinyin: raw.to_string(),
                weight: 9995,
            });
            return Some(cands);
        }
    }

    // Trường hợp 2: 8 số liền nhau YYYYMMDD (ví dụ: 20260826)
    if query.len() == 8 && query.chars().all(|c| c.is_ascii_digit()) {
        let year: u32 = query[0..4].parse().ok()?;
        let month: u32 = query[4..6].parse().ok()?;
        let day: u32 = query[6..8].parse().ok()?;

        if year >= 1000 && year <= 9999 && month >= 1 && month <= 12 && day >= 1 && day <= 31 {
            let mut cands = Vec::new();
            cands.push(Candidate {
                text: format!("{:04}年{:02}月{:02}日", year, month, day),
                pinyin: raw.to_string(),
                weight: 9999,
            });
            cands.push(Candidate {
                text: format!("{}年{}月{}日", format_year_chinese(year), number_to_chinese_lower(month as u64), number_to_chinese_lower(day as u64)),
                pinyin: raw.to_string(),
                weight: 9998,
            });
            cands.push(Candidate {
                text: format!("{:04}-{:02}-{:02}", year, month, day),
                pinyin: raw.to_string(),
                weight: 9997,
            });
            return Some(cands);
        }
    }

    // Trường hợp 3: MM.DD (hoặc M.D)
    if parts.len() == 2 {
        let month: u32 = parts[0].parse().ok()?;
        let day: u32 = parts[1].parse().ok()?;
        if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
            let mut cands = Vec::new();
            cands.push(Candidate {
                text: format!("{:02}月{:02}日", month, day),
                pinyin: raw.to_string(),
                weight: 9999,
            });
            cands.push(Candidate {
                text: format!("{}月{}日", number_to_chinese_lower(month as u64), number_to_chinese_lower(day as u64)),
                pinyin: raw.to_string(),
                weight: 9998,
            });
            cands.push(Candidate {
                text: format!("{}月{}日", month, day),
                pinyin: raw.to_string(),
                weight: 9997,
            });
            return Some(cands);
        }
    }

    None
}

fn format_year_chinese(year: u32) -> String {
    let digits = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    year.to_string()
        .chars()
        .map(|c| digits[c.to_digit(10).unwrap_or(0) as usize])
        .collect()
}

fn format_year_upper(year: u32) -> String {
    let digits = ['零', '壹', '贰', '叁', '肆', '伍', '陆', '柒', '捌', '玖'];
    year.to_string()
        .chars()
        .map(|c| digits[c.to_digit(10).unwrap_or(0) as usize])
        .collect()
}

fn digit_to_chinese_char_lower(c: char) -> char {
    match c {
        '0' => '零', '1' => '一', '2' => '二', '3' => '三', '4' => '四',
        '5' => '五', '6' => '六', '7' => '七', '8' => '八', '9' => '九',
        _ => c,
    }
}

fn digit_to_chinese_char_upper(c: char) -> char {
    match c {
        '0' => '零', '1' => '壹', '2' => '贰', '3' => '叁', '4' => '肆',
        '5' => '伍', '6' => '陆', '7' => '柒', '8' => '捌', '9' => '玖',
        _ => c,
    }
}

pub fn number_to_chinese_lower(mut n: u64) -> String {
    if n == 0 {
        return "零".to_string();
    }

    let digits = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    let section_units = ["", "万", "亿", "万亿"];
    let mut sections = Vec::new();

    while n > 0 {
        sections.push(n % 10000);
        n /= 10000;
    }

    let mut result = String::new();
    let mut need_zero = false;

    for (i, &sec) in sections.iter().enumerate().rev() {
        if sec != 0 {
            let mut sec_str = String::new();
            let s = sec;
            let d0 = (s / 1000) % 10;
            let d1 = (s / 100) % 10;
            let d2 = (s / 10) % 10;
            let d3 = s % 10;

            if d0 != 0 {
                sec_str.push_str(digits[d0 as usize]);
                sec_str.push('千');
            } else if need_zero {
                sec_str.push('零');
            }

            if d1 != 0 {
                sec_str.push_str(digits[d1 as usize]);
                sec_str.push('百');
            } else if !sec_str.is_empty() && (d2 != 0 || d3 != 0) && !sec_str.ends_with('零') {
                sec_str.push('零');
            }

            if d2 != 0 {
                // Trường hợp đặc biệt: 10-19 ở đầu câu đọc là "十" thay vì "一十"
                if d2 == 1 && d0 == 0 && d1 == 0 && sections.len() == 1 {
                    sec_str.push('十');
                } else {
                    sec_str.push_str(digits[d2 as usize]);
                    sec_str.push('十');
                }
            } else if !sec_str.is_empty() && d3 != 0 && !sec_str.ends_with('零') {
                sec_str.push('零');
            }

            if d3 != 0 {
                sec_str.push_str(digits[d3 as usize]);
            }

            result.push_str(&sec_str);
            result.push_str(section_units[i]);
            need_zero = false;
        } else if !result.is_empty() {
            need_zero = true;
        }
    }

    result
}

pub fn number_to_chinese_upper(mut n: u64) -> String {
    if n == 0 {
        return "零".to_string();
    }

    let digits = ["零", "壹", "贰", "叁", "肆", "伍", "陆", "柒", "捌", "玖"];
    let section_units = ["", "万", "亿", "万亿"];
    let mut sections = Vec::new();

    while n > 0 {
        sections.push(n % 10000);
        n /= 10000;
    }

    let mut result = String::new();
    let mut need_zero = false;

    for (i, &sec) in sections.iter().enumerate().rev() {
        if sec != 0 {
            let mut sec_str = String::new();
            let s = sec;
            let d0 = (s / 1000) % 10;
            let d1 = (s / 100) % 10;
            let d2 = (s / 10) % 10;
            let d3 = s % 10;

            if d0 != 0 {
                sec_str.push_str(digits[d0 as usize]);
                sec_str.push('仟');
            } else if need_zero {
                sec_str.push('零');
            }

            if d1 != 0 {
                sec_str.push_str(digits[d1 as usize]);
                sec_str.push('佰');
            } else if !sec_str.is_empty() && (d2 != 0 || d3 != 0) && !sec_str.ends_with('零') {
                sec_str.push('零');
            }

            if d2 != 0 {
                sec_str.push_str(digits[d2 as usize]);
                sec_str.push('拾');
            } else if !sec_str.is_empty() && d3 != 0 && !sec_str.ends_with('零') {
                sec_str.push('零');
            }

            if d3 != 0 {
                sec_str.push_str(digits[d3 as usize]);
            }

            result.push_str(&sec_str);
            result.push_str(section_units[i]);
            need_zero = false;
        } else if !result.is_empty() {
            need_zero = true;
        }
    }

    result
}

fn to_roman_numerals(mut num: u32) -> String {
    let lookup = [
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
    ];
    let mut roman = String::new();
    for &(val, sym) in &lookup {
        while num >= val {
            roman.push_str(sym);
            num -= val;
        }
    }
    roman
}

fn eval_math_expression(expr: &str) -> Option<f64> {
    let clean: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    if clean.is_empty() {
        return None;
    }

    // Đánh giá biểu thức đơn giản (hỗ trợ +, -, *, / và số thập phân)
    // Tách thành token
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();

    for c in clean.chars() {
        if c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            tokens.push(c.to_string());
        } else if c.is_ascii_digit() || c == '.' {
            current.push(c);
        } else {
            return None;
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    // Parser đệ quy đơn giản
    let mut pos = 0;
    parse_expr(&tokens, &mut pos)
}

fn parse_expr(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let mut lhs = parse_term(tokens, pos)?;
    while *pos < tokens.len() {
        let op = &tokens[*pos];
        if op == "+" {
            *pos += 1;
            let rhs = parse_term(tokens, pos)?;
            lhs += rhs;
        } else if op == "-" {
            *pos += 1;
            let rhs = parse_term(tokens, pos)?;
            lhs -= rhs;
        } else {
            break;
        }
    }
    Some(lhs)
}

fn parse_term(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let mut lhs = parse_factor(tokens, pos)?;
    while *pos < tokens.len() {
        let op = &tokens[*pos];
        if op == "*" {
            *pos += 1;
            let rhs = parse_factor(tokens, pos)?;
            lhs *= rhs;
        } else if op == "/" {
            *pos += 1;
            let rhs = parse_factor(tokens, pos)?;
            if rhs == 0.0 {
                return None;
            }
            lhs /= rhs;
        } else {
            break;
        }
    }
    Some(lhs)
}

fn parse_factor(tokens: &[String], pos: &mut usize) -> Option<f64> {
    if *pos >= tokens.len() {
        return None;
    }
    let tok = &tokens[*pos];
    if tok == "(" {
        *pos += 1;
        let val = parse_expr(tokens, pos)?;
        if *pos < tokens.len() && &tokens[*pos] == ")" {
            *pos += 1;
            Some(val)
        } else {
            None
        }
    } else if tok == "-" {
        *pos += 1;
        let val = parse_factor(tokens, pos)?;
        Some(-val)
    } else {
        *pos += 1;
        tok.parse::<f64>().ok()
    }
}
