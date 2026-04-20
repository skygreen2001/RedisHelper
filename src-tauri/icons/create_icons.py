import struct
import zlib

def crc32(data):
    crc = 0xffffffff
    table = [0] * 256
    for i in range(256):
        c = i
        for _ in range(8):
            c = (c >> 1) ^ 0xedb88320 if c & 1 else c >> 1
        table[i] = c
    for byte in data:
        crc = table[(crc ^ byte) & 0xff] ^ (crc >> 8)
    return crc ^ 0xffffffff

def create_png(width, height, filename):
    signature = b'\x89PNG\r\n\x1a\n'
    
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr = b'IHDR' + ihdr_data
    ihdr_crc = struct.pack('>I', crc32(ihdr) & 0xffffffff)
    ihdr_chunk = struct.pack('>I', 13) + ihdr + ihdr_crc
    
    # 简单的Redis logo图案 (红色机器人)
    raw_data = b''
    for y in range(height):
        raw_data += b'\x00'
        for x in range(width):
            # 红色背景
            if (x > width*0.2 and x < width*0.8 and y > height*0.1 and y < height*0.9):
                raw_data += b'\xff\x33\x33\xff'  # 红色
            else:
                raw_data += b'\xff\xff\xff\xff'  # 白色
    
    compressed = zlib.compress(raw_data)
    idat_data = compressed
    idat = b'IDAT' + idat_data
    idat_crc = struct.pack('>I', crc32(idat) & 0xffffffff)
    idat_chunk = struct.pack('>I', len(idat_data)) + idat + idat_crc
    
    iend = b'IEND'
    iend_crc = struct.pack('>I', crc32(iend) & 0xffffffff)
    iend_chunk = struct.pack('>I', 0) + iend + iend_crc
    
    with open(filename, 'wb') as f:
        f.write(signature + ihdr_chunk + idat_chunk + iend_chunk)

# 生成不同尺寸的图标
create_png(32, 32, '32x32.png')
create_png(128, 128, '128x128.png')
create_png(256, 256, '128x128@2x.png')
print('Icons created successfully')
