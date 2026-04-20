#!/usr/bin/env python3
"""
Tauri Icon Generator - 圆角版本
使用 Pillow 将 logo.png 生成为 Tauri 所需的全套 icon 文件，并添加圆角效果
"""
from PIL import Image, ImageDraw
import os
import struct

SRC = "/Library/WebServer/Documents/redis/src-tauri/icons/redis-logo.png"
DST = "/Library/WebServer/Documents/redis/src-tauri/icons"

# Tauri 标准 icon 列表（文件名 → 尺寸）
ICONS = {
    "32x32.png":      (32,  32),
    "128x128.png":    (128, 128),
    "128x128@2x.png": (256, 256),
    "icon.png":       (512, 512),
}

# macOS .icns 需要的尺寸
MACOS_SIZES = [16, 32, 64, 128, 256, 512, 1024]

def make_rounded(img, radius_ratio=0.25):
    """为图片添加圆角效果"""
    width, height = img.size
    radius = int(min(width, height) * radius_ratio)
    
    mask = Image.new('L', (width, height), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle([0, 0, width, height], radius=radius, fill=255)
    
    result = Image.new('RGBA', (width, height), (0, 0, 0, 0))
    result.paste(img, (0, 0), mask)
    return result

def make_icns(img_path, out_path):
    """生成 macOS .icns 文件（圆角版本）"""
    icns_types = {
        16:   b'icp4',
        32:   b'icp5',
        64:   b'icp6',
        128:  b'ic07',
        256:  b'ic08',
        512:  b'ic09',
        1024: b'ic10',
    }
    
    src = Image.open(img_path).convert("RGBA")
    
    icon_data = b''
    for size, type_tag in icns_types.items():
        resized = src.resize((size, size), Image.LANCZOS)
        rounded = make_rounded(resized)
        import io
        buf = io.BytesIO()
        rounded.save(buf, format='PNG')
        png_bytes = buf.getvalue()
        length = len(png_bytes) + 8
        icon_data += type_tag + struct.pack('>I', length) + png_bytes
    
    total = len(icon_data) + 8
    with open(out_path, 'wb') as f:
        f.write(b'icns' + struct.pack('>I', total) + icon_data)
    print(f"  ✅ icon.icns ({total} bytes)")

def main():
    print(f"📂 源文件: {SRC}")
    print(f"📂 输出目录: {DST}")
    print()
    
    if not os.path.exists(SRC):
        print(f"❌ 找不到源文件: {SRC}")
        return
    
    src_img = Image.open(SRC)
    print(f"🖼  原始尺寸: {src_img.width}x{src_img.height}, 模式: {src_img.mode}")
    
    img = src_img.convert("RGBA")
    print()
    
    # 生成各尺寸 PNG（圆角版本）
    print("🔨 生成圆角 PNG icons...")
    for filename, (w, h) in ICONS.items():
        out_path = os.path.join(DST, filename)
        resized = img.resize((w, h), Image.LANCZOS)
        rounded = make_rounded(resized)
        rounded.save(out_path, "PNG", optimize=True)
        size_kb = os.path.getsize(out_path) / 1024
        print(f"  ✅ {filename:<22} {w}x{h}  ({size_kb:.1f} KB)")
    
    # 生成 macOS .icns（圆角版本）
    print()
    print("🍎 生成 macOS icon.icns (圆角版本)...")
    icns_path = os.path.join(DST, "icon.icns")
    make_icns(SRC, icns_path)
    
    # 生成 Windows .ico（圆角版本）
    print()
    print("🪟 生成 Windows icon.ico (圆角版本)...")
    ico_sizes = [(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)]
    ico_images = [make_rounded(img.resize(s, Image.LANCZOS)) for s in ico_sizes]
    ico_path = os.path.join(DST, "icon.ico")
    ico_images[0].save(
        ico_path, format='ICO',
        sizes=ico_sizes,
        append_images=ico_images[1:]
    )
    size_kb = os.path.getsize(ico_path) / 1024
    print(f"  ✅ icon.ico  ({size_kb:.1f} KB, 含 {len(ico_sizes)} 种尺寸)")
    
    print()
    print("🎉 全部完成！生成文件列表:")
    for f in sorted(os.listdir(DST)):
        if not f.endswith('.py'):
            fpath = os.path.join(DST, f)
            size_kb = os.path.getsize(fpath) / 1024
            print(f"   {f:<28} {size_kb:>7.1f} KB")

if __name__ == "__main__":
    main()
