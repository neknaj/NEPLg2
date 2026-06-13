"use strict";

function divS(numerator, denominator) {
    return Math.trunc(numerator / denominator);
}

function u8(value) {
    return ((value % 256) + 256) % 256;
}

function mandelbrotCx(width, x) {
    return divS(x * 300, width - 1) - 220;
}

function mandelbrotCy(height, y) {
    return divS(y * 220, height - 1) - 110;
}

function mandelbrotEscapeIteration(width, height, limit, x, y) {
    const cx = mandelbrotCx(width, x);
    const cy = mandelbrotCy(height, y);
    let zx = 0;
    let zy = 0;
    let iter = 0;
    while (true) {
        const zx2 = zx * zx;
        const zy2 = zy * zy;
        if (zx2 + zy2 >= 40000 || iter >= limit) {
            return iter;
        }
        const nextZx = divS(zx2, 100) - divS(zy2, 100) + cx;
        const nextZy = divS(zx * zy * 2, 100) + cy;
        zx = nextZx;
        zy = nextZy;
        iter += 1;
    }
}

function createExpectedMandelbrotRgbaRow(width, height, limit) {
    return (y) => {
        const bytes = [];
        for (let x = 0; x < width; x += 1) {
            const iter = mandelbrotEscapeIteration(width, height, limit, x, y);
            if (iter === limit) {
                bytes.push(3, 7, 12, 255);
            } else {
                bytes.push(u8(24 + iter * 3), u8(52 + iter * 5), u8(98 + iter * 4), 255);
            }
        }
        return bytes;
    };
}

module.exports = {
    createExpectedMandelbrotRgbaRow,
};
