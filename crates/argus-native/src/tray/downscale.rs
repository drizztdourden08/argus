//! Makes a square picture smaller by averaging the pixels each new pixel covers. Whole integers,
//! so the result is the same on every machine. Pure, so it is tested without a window.
// test-tag: critical

use super::icon::Rgba;

/// A square `image` shrunk to `to` pixels a side. Each new pixel is the mean of the source area it
/// covers, weighted by how much of each source pixel falls inside it and by its alpha, so no
/// edge turns dark or light. A target of 0 or at least the current size gives the image as it is,
/// because this never enlarges.
#[must_use]
pub fn downscale(image: &Rgba, to: u32) -> Rgba {
    if to == 0 || to >= image.width {
        return image.clone();
    }
    let mut pixels = Vec::with_capacity(usize::try_from(to * to * 4).unwrap_or(0));
    for y in 0..u64::from(to) {
        for x in 0..u64::from(to) {
            pixels.extend(cell(image, x, y, u64::from(to)));
        }
    }
    Rgba {
        width: to,
        height: to,
        pixels,
    }
}

/// The pixel at (`x`, `y`) of the shrunk picture, `to` pixels a side.
fn cell(image: &Rgba, x: u64, y: u64, to: u64) -> [u8; 4] {
    let from = u64::from(image.width);
    let width = usize::try_from(from).unwrap_or(0);
    let mut sums = [0_u64; 3];
    let mut alphas = 0_u64;
    for sy in span(y, from, to) {
        let row = overlap(y, sy, from, to);
        for sx in span(x, from, to) {
            let at =
                (usize::try_from(sy).unwrap_or(0) * width + usize::try_from(sx).unwrap_or(0)) * 4;
            let weight = u64::from(image.pixels[at + 3]) * row * overlap(x, sx, from, to);
            alphas += weight;
            for (sum, channel) in sums.iter_mut().zip(&image.pixels[at..at + 3]) {
                *sum += u64::from(*channel) * weight;
            }
        }
    }
    let channel = |sum: u64| u8::try_from(sum.checked_div(alphas).unwrap_or(0)).unwrap_or(u8::MAX);
    let alpha = u8::try_from(alphas / (from * from)).unwrap_or(u8::MAX);
    [channel(sums[0]), channel(sums[1]), channel(sums[2]), alpha]
}

/// The source rows (or columns) that new row `index` touches.
fn span(index: u64, from: u64, to: u64) -> std::ops::Range<u64> {
    (index * from / to)..((index + 1) * from).div_ceil(to)
}

/// How much of source pixel `source` lies inside new pixel `index`, in units of `1 / (from * to)`
/// of the picture side. The overlaps of one new pixel add up to `from`.
fn overlap(index: u64, source: u64, from: u64, to: u64) -> u64 {
    let end = ((index + 1) * from).min((source + 1) * to);
    let start = (index * from).max(source * to);
    end.saturating_sub(start)
}

#[cfg(test)]
mod tests {
    use super::{Rgba, downscale};

    fn square(side: u32, make: impl Fn(u32, u32) -> [u8; 4]) -> Rgba {
        let mut pixels = Vec::new();
        for y in 0..side {
            for x in 0..side {
                pixels.extend(make(x, y));
            }
        }
        Rgba {
            width: side,
            height: side,
            pixels,
        }
    }

    #[test]
    fn a_solid_picture_stays_solid_at_any_smaller_size() {
        let solid = square(48, |_, _| [255, 255, 255, 255]);
        for to in [16, 20, 24, 36, 40] {
            let small = downscale(&solid, to);
            assert_eq!((small.width, small.height), (to, to));
            assert!(
                small
                    .pixels
                    .chunks_exact(4)
                    .all(|p| p == [255, 255, 255, 255])
            );
        }
    }

    #[test]
    fn half_the_size_averages_each_block_of_four() {
        let halves = square(4, |x, _| {
            if x < 2 {
                [0, 0, 0, 255]
            } else {
                [255, 255, 255, 255]
            }
        });
        let small = downscale(&halves, 2);
        assert_eq!(
            small.pixels,
            [
                0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255
            ]
        );
        let mixed = square(2, |x, y| {
            if x == y {
                [200, 0, 0, 255]
            } else {
                [0, 0, 0, 0]
            }
        });
        assert_eq!(downscale(&mixed, 1).pixels, [200, 0, 0, 127]);
    }

    #[test]
    fn a_clear_pixel_does_not_darken_the_colour_of_its_neighbours() {
        let edge = square(4, |x, _| {
            if x < 2 {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 0]
            }
        });
        let small = downscale(&edge, 1);
        assert_eq!(small.pixels, [255, 255, 255, 127]);
    }

    #[test]
    fn it_never_enlarges() {
        let image = square(16, |_, _| [1, 2, 3, 4]);
        assert_eq!(downscale(&image, 16), image);
        assert_eq!(downscale(&image, 64), image);
        assert_eq!(downscale(&image, 0), image);
    }
}
