# 🚀 VCKeys 3.0.8 Pro Native (Tauri v2 + Rust)

Bộ gõ Đa Ngôn Ngữ Native siêu tốc (**Việt - Trung - Anh**) viết hoàn toàn bằng **Rust** kết hợp **Tauri v2** và giao diện **Modern Fluent UI**.

[![Release](https://img.shields.io/github/v/release/tahiep2012/VCKey?style=flat-square&color=0078D4)](https://github.com/tahiep2012/VCKey/releases)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue?style=flat-square&logo=tauri)](https://tauri.app/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

---

## ✨ Điểm Nổi Bật Vượt Trội

### 1. ⚡ Hiệu năng & Tốc độ đỉnh cao
- Viết bằng **Rust Native** kết hợp nhân xử lý bàn phím Win32 Low-Level Hook, độ trễ phản hồi phím $< 0.01\text{ ms}$.
- Mức chiếm dụng RAM cực thấp: **~15 - 25 MB** RAM (nhỏ hơn nhiều lần so với các bộ gõ Electron/Python).
- Chạy mượt mà, ổn định trên Windows 10 và Windows 11.

---

### 2. 🇻🇳 Bộ gõ Tiếng Việt Telex Chuẩn UniKey / EVKey
- **Tích hợp nhân C++ UniKey Engine** chính gốc của tác giả Phạm Kim Long kết hợp engine Rust mở rộng.
- Hỗ trợ đầy đủ các kiểu gõ thông dụng: **Telex**, **VNI**, **Simple Telex**, **VIQR**.
- Hỗ trợ đa dạng bảng mã: **Unicode**, **TCVN3 (ABC)**, **VNI Windows**, **VIQR**...
- **Quy tắc chuẩn hóa hủy dấu thanh Telex & lặp modifier kép (v3.0.6)**:
  - **1 phím modifier**: Tạo dấu thanh tiếng Việt (`p a s` $\rightarrow$ `pá`, `c a s` $\rightarrow$ `cá`, `k i s` $\rightarrow$ `kí`, `l e s` $\rightarrow$ `lé`, `b o s` $\rightarrow$ `bó`, `d i f` $\rightarrow$ `dì`).
  - **2 phím modifier**: Hủy dấu thanh, trở về ký tự không dấu (`p a s s` $\rightarrow$ `pas`, `c a s s` $\rightarrow$ `cas`, `k i s s` $\rightarrow$ `kis`, `l e s s` $\rightarrow$ `les`, `b o s s` $\rightarrow$ `bos`, `d i f f` $\rightarrow$ `dif`).
  - **3 phím modifier**: Thêm lại phụ âm thứ hai (`p a s s s` $\rightarrow$ `pass`, `c a s s s` $\rightarrow$ `cass`, `k i s s s` $\rightarrow$ `kiss`, `l e s s s` $\rightarrow$ `less`, `b o s s s` $\rightarrow$ `boss`, `d i f f f` $\rightarrow$ `diff`).
- **Khắc phục triệt để lỗi nuốt phím / "dấu ẩn"** khi gõ các từ tiếng Anh không mang dấu (`releass`, `serverr`, `statuss`, `diffs`, `focuss`...).
- Bỏ dấu & tạo mũ tự do ở cuối từ (FreeMark: `dudocjw` $\rightarrow$ `được`, `deerd` $\rightarrow$ `để`).
- Cơ chế gõ phím thứ 3 hủy mũ (`ooo` $\rightarrow$ `oo` $\rightarrow$ `good`, `tooo` $\rightarrow$ `too`, `leee` $\rightarrow$ `lee`).
- **Tự động nhận diện và bảo toàn từ tiếng Anh** với từ điển tích hợp hơn 35.000 từ.
- Đặt dấu thanh mới chuẩn tiếng Việt hiện đại (TCVN 2002 / UniKey modern: `hóa`, `thủy`) hoặc kiểu truyền thống (`hoá`, `thuỷ`).
- **Tối ưu xóa khoảng trắng và bỏ dấu hồi tố liên từ (v3.0.7)**: Loại bỏ hoàn toàn idle timeout, cho phép người dùng dừng gõ tùy ý mà vẫn xóa dấu cách và bỏ dấu bình thường (`CO ` $\rightarrow$ chờ $\rightarrow$ Backspace $\rightarrow$ `S` $\rightarrow$ `CÓ`).
- **Nâng dung lượng bộ nhớ đệm từ đang gõ lên 15 ký tự (v3.0.8)**: Mở rộng ngưỡng tự phục hồi đệm từ 9 lên 15 ký tự, hỗ trợ tối đa cho các từ ghép dài, từ ngoại lai và thuật ngữ kỹ thuật mà không bị reset đệm sớm.

---

### 3. 🇨🇳 Bộ gõ Tiếng Trung Bính Âm Sogou Pinyin (搜狗输入法)
- **Từ điển bính âm khổng lồ hơn 457.000 từ**, tự động nạp ngầm (pre-warm) giúp gõ tức thì ngay từ ký tự đầu tiên.
- **Ghép câu dài tự động (Dynamic Sentence Composition)**: Gõ cả cụm dài liền mạch không cần ngắt quãng.
- **Cửa sổ ứng viên (Candidate Popup Window)** thiết kế chuẩn Win32 Native, hiển thị dấu thanh bính âm trực quan (`ā á ǎ à`, `ō ó ǒ ò`, `ē é ě è`, `ī í ǐ ì`, `ū ú ǔ ù`, `ǖ ǘ ǚ ǜ`).
- **Từ điển người dùng thông minh (User Dictionary)**: Tự động ghi nhớ và ưu tiên các từ, cụm từ hay gõ lên vị trí số 1.
- Hỗ trợ gõ tắt bính âm (Jianpin: `nh` $\rightarrow$ `你好`, `bj` $\rightarrow$ `北京`, `dd` $\rightarrow$ `等等`).
- Chuyển đổi nhanh Giản thể $\leftrightarrow$ Phồn thể (`Ctrl + Shift + F`).
- Gợi ý biểu tượng cảm xúc Emoji thông minh theo ngữ cảnh.

---

### 4. 🇬🇧 Chế độ Tiếng Anh & Chuyển Đổi Nhanh
- **Chuyển đổi 3 ngôn ngữ tức thì (`Ctrl + Shift` hoặc `Alt + Z`)**:
  - 🇻🇳 Tiếng Việt $\leftrightarrow$ 🇨🇳 Tiếng Trung $\leftrightarrow$ 🇬🇧 Tiếng Anh.
- **OSD Toast**: Thông báo nổi tuyệt đẹp trên góc màn hình khi chuyển đổi chế độ.
- Biểu tượng khay hệ thống (System Tray) tự động thay đổi theo ngôn ngữ (`V`, `中`, `E`).

---

### 5. 🎨 Giao diện Modern Fluent UI
- 5 Tabs chức năng trực quan: **Bộ gõ**, **Tiếng Trung**, **Gõ tắt Macro**, **Hệ thống**, **Thông tin**.
- Thu gọn xuống khay hệ thống (System Tray), không chiếm diện tích Taskbar.
- Tùy chọn tự khởi động cùng Windows.
- Single Instance: Không bao giờ bị chạy trùng lặp nhiều tiến trình.

---

## 📦 Tải Về & Cài Đặt (Portable)

Tải phiên bản mới nhất tại: **[GitHub Releases](https://github.com/tahiep2012/VCKey/releases)**

| Tệp tải về | Mô tả |
| :--- | :--- |
| **`VCKeys.zip`** *(Khuyên dùng)* | Bản Portable trọn gói đầy đủ thư viện phụ thuộc (`libc++.dll`, `libunwind.dll`, `WebView2Loader.dll`) và cấu hình sẵn. Giải nén và chạy ngay! |
| **`VCKeys.exe`** | File thực thi chính chạy trực tiếp. |

---

## 🛠️ Biên Dịch & Chạy Kiểm Thử (Build & Test)

### Yêu cầu môi trường
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (1.80+)
- [Node.js](https://nodejs.org/) & [Tauri CLI v2](https://tauri.app/)
- C++ Build Tools (MSVC trên Windows)

### Chạy kiểm thử tự động
```bash
cd src-tauri
cargo test
```

### Biên dịch bản phát hành Release siêu tối ưu
```bash
cd src-tauri
cargo build --release
```
File thực thi sau khi biên dịch nằm tại: `src-tauri/target/release/vckeys-tauri.exe`.

---

## 📄 Bản Quyền & Tác Giả (License)

- Phát triển bởi **Võ Tá Hiệp** ([@tahiep2012](https://github.com/tahiep2012)).
- Tích hợp nhân C++ **UniKey Engine** của tác giả **Phạm Kim Long** theo giấy phép mã nguồn mở.
- Giấy phép phân phối: **MIT License**.
