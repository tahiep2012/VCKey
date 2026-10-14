import os
from PIL import Image, ImageDraw, ImageFont

def create_v_icon(size, radius_ratio=0.22, is_tray=False):
    scale = 4
    s = size * scale
    
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    radius = int(s * radius_ratio)
    
    mask = Image.new("L", (s, s), 0)
    mask_draw = ImageDraw.Draw(mask)
    margin = int(s * 0.02) if not is_tray else 0
    mask_draw.rounded_rectangle(
        [margin, margin, s - 1 - margin, s - 1 - margin],
        radius=radius,
        fill=255
    )
    
    bg = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    for y in range(s):
        ratio = y / s
        r = int(239 * (1 - ratio) + 200 * ratio)
        g = int(50 * (1 - ratio) + 25 * ratio)
        b = int(50 * (1 - ratio) + 25 * ratio)
        bg_line = Image.new("RGBA", (s, 1), (r, g, b, 255))
        bg.paste(bg_line, (0, y))
    
    img.paste(bg, (0, 0), mask)
    
    draw = ImageDraw.Draw(img)
    font_size = int(s * 0.68)
    font = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", font_size)
    
    bbox = draw.textbbox((0, 0), "V", font=font)
    text_w = bbox[2] - bbox[0]
    text_h = bbox[3] - bbox[1]
    
    tx = (s - text_w) / 2 - bbox[0]
    ty = (s - text_h) / 2 - bbox[1] - int(s * 0.01)
    
    draw.text((tx, ty), "V", fill=(255, 255, 255, 255), font=font)
    
    final_img = img.resize((size, size), Image.Resampling.LANCZOS)
    return final_img

icons_dir = "d:/Antigravity/VCKey-Tauri/src-tauri/icons"

icon_512 = create_v_icon(512)
icon_512.save(os.path.join(icons_dir, "icon.png"), "PNG")

icon_256 = create_v_icon(256)
icon_256.save(os.path.join(icons_dir, "128x128@2x.png"), "PNG")

icon_128 = create_v_icon(128)
icon_128.save(os.path.join(icons_dir, "128x128.png"), "PNG")

icon_32 = create_v_icon(32)
icon_32.save(os.path.join(icons_dir, "32x32.png"), "PNG")

tray_32 = create_v_icon(32, radius_ratio=0.18, is_tray=True)
tray_32.save(os.path.join(icons_dir, "tray_vn.png"), "PNG")

tray_ico_img = create_v_icon(64, radius_ratio=0.18, is_tray=True)
tray_ico_img.save(os.path.join(icons_dir, "tray_vn.ico"), format="ICO", sizes=[(16, 16), (24, 24), (32, 32), (48, 48)])

ico_sizes = [(16, 16), (20, 20), (24, 24), (32, 32), (40, 40), (48, 48), (64, 64), (128, 128), (256, 256)]
icon_512.save(
    os.path.join(icons_dir, "icon.ico"),
    format="ICO",
    sizes=ico_sizes
)

print("All red 'V' icons generated successfully!")
