import zlib
import struct
import os

def write_png(filename, width, height, rgba_data):
    png = bytearray(b'\x89PNG\r\n\x1a\n')
    
    # IHDR chunk
    ihdr_data = struct.pack('!IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data)
    png += struct.pack('!I', len(ihdr_data)) + b'IHDR' + ihdr_data + struct.pack('!I', ihdr_crc)
    
    # IDAT chunk
    raw_lines = bytearray()
    for y in range(height):
        raw_lines.append(0) # Filter type 0
        start = y * width * 4
        raw_lines.extend(rgba_data[start:start + width * 4])
    
    idat_compressed = zlib.compress(bytes(raw_lines))
    idat_crc = zlib.crc32(b'IDAT' + idat_compressed)
    png += struct.pack('!I', len(idat_compressed)) + b'IDAT' + idat_compressed + struct.pack('!I', idat_crc)
    
    # IEND chunk
    iend_crc = zlib.crc32(b'IEND')
    png += struct.pack('!I', 0) + b'IEND' + struct.pack('!I', iend_crc)
    
    os.makedirs(os.path.dirname(filename), exist_ok=True)
    with open(filename, 'wb') as f:
        f.write(png)

# Generate 128x128 RGBA image
w, h = 128, 128
rgba = bytearray()

for y in range(h):
    for x in range(w):
        dx = (16 - x) if x < 16 else ((x - 111) if x >= 112 else 0)
        dy = (16 - y) if y < 16 else ((y - 111) if y >= 112 else 0)
        is_corner = (dx * dx + dy * dy) > 256
        
        if is_corner:
            rgba.extend([0, 0, 0, 0])
            continue
            
        # Gold Clip handle (y: 8..24, x: 48..80)
        if 8 <= y <= 24 and 48 <= x <= 80:
            if 14 <= y <= 18 and 60 <= x <= 68:
                rgba.extend([0x12, 0x12, 0x14, 0xFF])
            else:
                rgba.extend([0xF5, 0x9E, 0x0B, 0xFF])
            continue
            
        # Clipboard Board (x: 24..104, y: 20..116)
        if 24 <= x <= 104 and 20 <= y <= 116:
            if x == 24 or x == 104 or y == 20 or y == 116:
                rgba.extend([0x3D, 0x3D, 0x47, 0xFF])
                continue
            if 40 <= y <= 46 and 40 <= x <= 88:
                rgba.extend([0xF5, 0xF5, 0xF7, 0xFF])
            elif 60 <= y <= 66 and 40 <= x <= 76:
                rgba.extend([0xA1, 0xA1, 0xAA, 0xFF])
            elif 80 <= y <= 86 and 40 <= x <= 84:
                rgba.extend([0xA1, 0xA1, 0xAA, 0xFF])
            elif 98 <= y <= 102 and 40 <= x <= 64:
                rgba.extend([0x71, 0x71, 0x7A, 0xFF])
            else:
                rgba.extend([0x1F, 0x1F, 0x24, 0xFF])
            continue
            
        rgba.extend([0x12, 0x12, 0x14, 0xFF])

script_dir = os.path.dirname(os.path.abspath(__file__))
out_file = os.path.join(script_dir, 'rusty-clipboard.png')
write_png(out_file, w, h, rgba)
print("Generated PNG icon at", out_file)
