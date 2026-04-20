#!/usr/bin/env python3
"""
将图标转换为圆角版本
"""
from PIL import Image

def make_rounded_icon(input_path, output_path, radius_ratio=0.2):
    """
    将图标转换为圆角版本
    :param input_path: 输入图标路径
    :param output_path: 输出图标路径
    :param radius_ratio: 圆角半径比例（0-0.5）
    """
    img = Image.open(input_path).convert("RGBA")
    width, height = img.size
    
    # 计算圆角半径
    radius = int(min(width, height) * radius_ratio)
    
    # 创建圆角遮罩
    mask = Image.new('L', (width, height), 0)
    draw = ImageDraw.Draw(mask)
    
    # 绘制圆角矩形
    draw.rounded_rectangle([0, 0, width, height], radius=radius, fill=255)
    
    # 应用遮罩
    result = Image.new('RGBA', (width, height), (0, 0, 0, 0))
    result.paste(img, (0, 0), mask)
    
    result.save(output_path, 'PNG', optimize=True)
    print(f"✅ 圆角图标已生成: {output_path}")

if __name__ == "__main__":
    from PIL import ImageDraw
    
    # 处理主要图标
    input_icon = "/Library/WebServer/Documents/redis/src-tauri/icons/icon.png"
    output_icon = "/Library/WebServer/Documents/redis/src-tauri/icons/icon-rounded.png"
    
    print(f"🎨 正在生成圆角图标...")
    make_rounded_icon(input_icon, output_icon, radius_ratio=0.25)
    
    # 也处理其他尺寸
    icons_to_process = [
        "32x32.png",
        "128x128.png",
        "128x128@2x.png",
    ]
    
    for icon in icons_to_process:
        input_path = f"/Library/WebServer/Documents/redis/src-tauri/icons/{icon}"
        output_path = f"/Library/WebServer/Documents/redis/src-tauri/icons/{icon.replace('.png', '-rounded.png')}"
        make_rounded_icon(input_path, output_path, radius_ratio=0.25)
