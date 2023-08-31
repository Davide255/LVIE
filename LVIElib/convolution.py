from math import ceil
import numpy
from PIL import Image

def split(image):
    r_buf = []
    g_buf = []
    b_buf = []
    
    for row in image:
        r_row = []
        g_row = []
        b_row = []

        for pix in row:
            r_row.append(pix[0])
            g_row.append(pix[1])
            b_row.append(pix[2])

        r_buf.append(r_row)
        g_buf.append(g_row)
        b_buf.append(b_row)
    
    return r_buf, g_buf, b_buf



def conv(buf, kernel):
    (buf, kernel) = numpy.matrix(buf), numpy.matrix(kernel)

    (bh, bw) = buf.shape
    (kh, kw) = kernel.shape
    #kernel = numpy.pad(kernel, (((bh-kh)//2, ceil((bh-kh)/2)), ((bw-kw)//2, ceil((bw-kw)/2))))
    kernel = numpy.pad(kernel, ((0, bh-kh), (0, bw-kw)))

    f_buf = numpy.fft.fft2(buf)
    f_kernel = numpy.fft.fft2(kernel)

    return numpy.rint(numpy.real(numpy.fft.ifft2(f_buf*f_kernel)))


def apply_conv(image, kernel):
    (r, g, b) = split(image)

    print("Splitted!")

    r = conv(r, kernel)
    g = conv(g, kernel)
    b = conv(b, kernel)

    print("Convolved!")
    
    buf = []
    for i, r_row in enumerate(r):
        buf_row = []
        for j, r_pix in enumerate(r_row):
            buf_row.append([r_pix, g[i][j], b[i][j]])
        buf.append(buf_row)

    print("Reconstructed")
    
    array = numpy.array(buf)
    print("Converted to array")
    return array


image = numpy.asarray(Image.open("/home/bolli/Desktop/DSCF0003.JPG"))
kernel = numpy.matrix('0 -1 0; -1 5 -1; 0 -1 0')
Image.fromarray(apply_conv(image, kernel).astype('uint8')).save('/home/bolli/Desktop/output.png')
