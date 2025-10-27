import struct

def create_png(filename, size, color_rgb=(0, 123, 255)):
    """Create a simple solid color PNG file"""
    # PNG signature
    png_signature = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk (image header)
    ihdr_data = struct.pack('>IIBBBBB', size, size, 8, 2, 0, 0, 0)
    ihdr_chunk = b'IHDR' + ihdr_data
    ihdr_crc = struct.pack('>I', 0xFFFFFFFF & ~binascii.crc32(ihdr_chunk))
    ihdr = struct.pack('>I', len(ihdr_data)) + ihdr_chunk + ihdr_crc
    
    # Create image data
    scanlines = []
    for y in range(size):
        scanline = bytes([0])  # Filter type
        for x in range(size):
            scanline += bytes(color_rgb)
        scanlines.append(scanline)
    
    raw_data = b''.join(scanlines)
    import zlib
    compressed = zlib.compress(raw_data, 9)
    
    idat_chunk = b'IDAT' + compressed
    idat_crc = struct.pack('>I', 0xFFFFFFFF & ~binascii.crc32(idat_chunk))
    idat = struct.pack('>I', len(compressed)) + idat_chunk + idat_crc
    
    # IEND chunk
    iend = struct.pack('>I', 0) + b'IEND' + struct.pack('>I', 0xAE426082)
    
    with open(filename, 'wb') as f:
        f.write(png_signature + ihdr + idat + iend)

import binascii
create_png('32x32.png', 32)
create_png('128x128.png', 128)
create_png('128x128@2x.png', 256)
print("Created PNG icons!")
