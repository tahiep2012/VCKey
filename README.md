# VCKey 2.0.5 Pro Native (Tauri v2 + Rust)

Bộ gõ Đa Ngôn Ngữ Native siêu tốc (Việt - Trung - Anh) viết hoàn toàn bằng **Rust** kết hợp **Tauri v2** và giao diện **Modern Fluent UI**.

---

## ✨ Điểm Nổi Bật Vượt Trội

1. **Hiệu năng & Tốc độ đỉnh cao**:
   - Viết bằng **Rust Native**, thời gian phản hồi phím $< 0.01\text{ ms}$.
   - Dung lượng file thực thi chỉ **~4.5 MB** (nhỏ hơn 7 lần so với bản Python 31.5 MB).
   - Mức chiếm dụng RAM cực thấp: **~15 - 20 MB** RAM.
2. **Bộ gõ Tiếng Việt Telex Chuẩn & Đầy Đủ**:
   - Bỏ dấu & tạo mũ tự do ở cuối từ (FreeMark: `dudocjw` $\rightarrow$ `được`, `deerd` $\rightarrow$ `để`).
   - Cơ chế gõ phím thứ 3 hủy mũ (`ooo` $\rightarrow$ `oo` $\rightarrow$ gõ `g o o o d` ra ngay `good`, `tooo` $\rightarrow$ `too`, `leee` $\rightarrow$ `lee`).
   - Thoát dấu nhanh bằng gõ đúp (`showw` $\rightarrow$ `show`, `ww` $\rightarrow$ `w`, `tess` $\rightarrow$ `tes`).
   - Tự động nhận diện và bảo toàn từ tiếng Anh (`good bye`, `book`, `food`, `cool`...).
   - Chuẩn đặt dấu thanh mới (TCVN 2002 / UniKey modern: `hóa`, `thủy`).
3. **Bộ gõ Tiếng Trung Bính Âm Sogou Pinyin (搜狗输入法)**:
   - Ghép câu dài tự động (Dynamic Sentence Composition).
   - Chuyển đổi Giản thể $\leftrightarrow$ Phồn thể (`Ctrl + Shift + F`).
   - Gợi ý biểu tượng cảm xúc Emoji.
4. **Chuyển đổi ngôn ngữ tức thì (`Ctrl + Shift`)**:
   - Vòng lặp 3 ngôn ngữ: 🇻🇳 Tiếng Việt $\leftrightarrow$ 🇨🇳 Tiếng Trung $\leftrightarrow$ 🇬🇧 Tiếng Anh.
5. **Giao diện Modern Fluent UI**:
   - 5 Tabs chức năng: Bộ gõ, Tiếng Trung, Gõ tắt Macro, Hệ thống, Thông tin.
   - Hỗ trợ khay hệ thống (System Tray).

---

## 🚀 Khởi Chạy Nhanh

- Chạy trực tiếp file [`chay_thu.bat`](file:///d:/Antigravity/VCKey-Tauri/chay_thu.bat) hoặc file thực thi [`src-tauri\target\release\vckey-tauri.exe`](file:///d:/Antigravity/VCKey-Tauri/src-tauri/target/release/vckey-tauri.exe).

---

## 🛠️ Biên Dịch & Chạy Kiểm Thử (Build & Test)

```bash
# Chạy toàn bộ Unit Tests
cd src-tauri
cargo test

# Biên dịch bản Release siêu tối ưu
cargo build --release
```
