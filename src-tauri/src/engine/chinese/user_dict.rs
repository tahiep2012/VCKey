use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use crate::engine::chinese::Candidate;

/// Lưu trữ từ điển cá nhân của người dùng (pinyin → danh sách chữ Hán ưu tiên).
/// Dữ liệu được ghi ra file JSON trong thư mục config của ứng dụng.
pub struct UserDict {
    /// pinyin (không dấu, in thường) → [chữ Hán ưu tiên, ...]
    pub entries: HashMap<String, Vec<String>>,
    pub path: PathBuf,
}

impl UserDict {
    /// Tải từ điển cá nhân từ file JSON. Tạo mới nếu file chưa tồn tại.
    pub fn load(path: PathBuf) -> Self {
        let entries = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    serde_json::from_str::<HashMap<String, Vec<String>>>(&content)
                        .unwrap_or_default()
                }
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        Self { entries, path }
    }

    /// Lưu từ điển cá nhân ra file JSON.
    pub fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
            let _ = fs::write(&self.path, json);
        }
    }

    /// Thêm hoặc cập nhật một cụm từ vào từ điển cá nhân.
    ///
    /// - `pinyin`: phiên âm không dấu, in thường (ví dụ: "cangkubu")
    /// - `text`:   chữ Hán tương ứng (ví dụ: "仓库部")
    ///
    /// Từ mới nhất được đặt lên đầu danh sách (ưu tiên gợi ý trước).
    /// Danh sách được giới hạn tối đa 10 ứng viên để tránh phình file.
    pub fn add_phrase(&mut self, pinyin: &str, text: &str) {
        let key = pinyin.to_lowercase().replace('\'', "");
        let list = self.entries.entry(key).or_insert_with(Vec::new);

        // Xóa nếu đã tồn tại (để đẩy lên đầu)
        list.retain(|x| x != text);

        // Chèn lên đầu (ưu tiên cao nhất)
        list.insert(0, text.to_string());

        // Giữ tối đa 10 ứng viên
        list.truncate(10);
    }

    /// Tra từ điển cá nhân và trả về danh sách ứng viên với weight cao nhất (9999).
    /// Trả về Vec rỗng nếu không tìm thấy.
    pub fn get_candidates(&self, pinyin: &str) -> Vec<Candidate> {
        let key = pinyin.to_lowercase().replace('\'', "");
        match self.entries.get(&key) {
            Some(list) => list
                .iter()
                .enumerate()
                .map(|(i, text)| Candidate {
                    text: text.clone(),
                    pinyin: key.clone(),
                    weight: 9999usize.saturating_sub(i),
                })
                .collect(),
            None => Vec::new(),
        }
    }

    /// Kiểm tra xem từ điển có rỗng không.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dict() -> UserDict {
        UserDict {
            entries: HashMap::new(),
            path: PathBuf::from("test_user_dict.json"),
        }
    }

    #[test]
    fn test_add_phrase_and_lookup() {
        let mut d = make_temp_dict();
        d.add_phrase("cangkubu", "仓库部");
        let cands = d.get_candidates("cangkubu");
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].text, "仓库部");
        assert_eq!(cands[0].weight, 9999);
    }

    #[test]
    fn test_dedup_and_priority() {
        let mut d = make_temp_dict();
        d.add_phrase("bu", "部");
        d.add_phrase("bu", "不");
        d.add_phrase("bu", "部"); // 重复 → đẩy "部" lên đầu
        let cands = d.get_candidates("bu");
        assert_eq!(cands[0].text, "部");
        assert_eq!(cands[1].text, "不");
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let mut d = make_temp_dict();
        d.add_phrase("CangKuBu", "仓库部");
        let cands = d.get_candidates("cangkubu");
        assert_eq!(cands[0].text, "仓库部");
    }

    #[test]
    fn test_missing_key_returns_empty() {
        let d = make_temp_dict();
        assert!(d.get_candidates("notexist").is_empty());
    }
}
