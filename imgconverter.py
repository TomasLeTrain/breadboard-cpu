from PIL import Image


ver_res = 100
hor_res = 160

image = Image.open("finch.png").convert("RGB")
out_file = open("finch.bin", "wb")

rb_lut = [85 * round(i * (3.0 / 255.0)) for i in range(256)]
g_lut =  [36 * round(i * (7.0 / 255.0)) for i in range(256)]

r, g, b = image.split()
r = r.point(rb_lut)
g = g.point(g_lut)
b = b.point(rb_lut)
#
out = Image.merge("RGB", (r, g, b))

pixels = image.load()

DRAWING_BIT = 0b00000100
NO_DRAWING_MASK = ~DRAWING_BIT

def makePixel(red,green,blue):
    red = int(round(red / 85)) & 0b11
    green = int(round(green / 36)) & 0b111
    blue = int(round(blue / 85)) & 0b11

    return NO_DRAWING_MASK & (red | (blue << 3) | (green << 5))

for y in range(ver_res):
    for x in range(hor_res):
        try:
            # print(pixels[x, y])
            r,g,b = pixels[x, y]
            # print(r,g,b)
            # print("{:08b}".format(makePixel(r,g,b)))
            out_file.write(makePixel(r,g,b).to_bytes(1))
        except IndexError:
            out_file.write(chr(0))
