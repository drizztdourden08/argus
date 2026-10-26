//! The tray icons as pixels: the Argus eye for a running server, and the same eye greyed and
//! faded with a red badge for every other state. Pure, so it is tested without a window.

use std::io::Cursor;

use ico::IconDir;

use super::error::TrayError;

const ARGUS_ICO: &[u8] = include_bytes!("../../assets/argus.ico");
const SIZE: u32 = 32;
const SAMPLES: i64 = 4;
const RED: [u8; 3] = [220, 38, 38];
const WHITE: [u8; 3] = [255, 255, 255];

/// Which of the two icons to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    /// The server runs: the icon as it is.
    Running,
    /// The server is down or not known yet: grey, faded, with a red badge.
    Stopped,
}

/// A picture of straight (not premultiplied) RGBA pixels, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Four bytes per pixel.
    pub pixels: Vec<u8>,
}

/// The icon of Argus for a state, 32 by 32 pixels, from the icon file in the crate.
///
/// # Errors
///
/// `Platform` when the embedded icon file has no usable 32 pixel image.
pub fn render(kind: IconKind) -> Result<Rgba, TrayError> {
    let base = decode(ARGUS_ICO, SIZE)?;
    Ok(match kind {
        IconKind::Running => base,
        IconKind::Stopped => stopped(&base),
    })
}

fn decode(bytes: &[u8], size: u32) -> Result<Rgba, TrayError> {
    let fail = |why: String| TrayError::Platform(format!("the Argus icon is unusable: {why}"));
    let dir = IconDir::read(Cursor::new(bytes)).map_err(|e| fail(e.to_string()))?;
    let entry = dir
        .entries()
        .iter()
        .find(|entry| entry.width() == size && entry.height() == size)
        .ok_or_else(|| fail(format!("no {size} pixel image")))?;
    let image = entry.decode().map_err(|e| fail(e.to_string()))?;
    Ok(Rgba {
        width: image.width(),
        height: image.height(),
        pixels: image.rgba_data().to_vec(),
    })
}

fn stopped(base: &Rgba) -> Rgba {
    let mut out = base.clone();
    for pixel in out.pixels.chunks_exact_mut(4) {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]].map(u32::from);
        let grey = u8::try_from((30 * r + 59 * g + 11 * b) / 100).unwrap_or(u8::MAX);
        let faded = u8::try_from(a * 6 / 10).unwrap_or(u8::MAX);
        pixel.copy_from_slice(&[grey, grey, grey, faded]);
    }
    let size = i64::from(out.width);
    let center = size * 3 / 4;
    paint_disc(&mut out, center, size / 4, WHITE);
    paint_disc(&mut out, center, size * 3 / 16, RED);
    out
}

fn paint_disc(image: &mut Rgba, center: i64, radius: i64, color: [u8; 3]) {
    let width = usize::try_from(image.width).unwrap_or(0);
    for (index, pixel) in image.pixels.chunks_exact_mut(4).enumerate() {
        let x = i64::try_from(index % width).unwrap_or(0);
        let y = i64::try_from(index / width).unwrap_or(0);
        let coverage = disc_coverage(x - center, y - center, radius);
        if coverage > 0 {
            over(pixel, color, coverage);
        }
    }
}

fn disc_coverage(dx: i64, dy: i64, radius: i64) -> u32 {
    let step = 2 * SAMPLES;
    let mut inside = 0;
    for sy in 0..SAMPLES {
        for sx in 0..SAMPLES {
            let (px, py) = (
                dx * step + 2 * sx + 1 - SAMPLES,
                dy * step + 2 * sy + 1 - SAMPLES,
            );
            let reach = radius * step;
            inside += u32::from(px * px + py * py <= reach * reach);
        }
    }
    inside * 255 / u32::try_from(SAMPLES * SAMPLES).unwrap_or(16)
}

fn over(pixel: &mut [u8], color: [u8; 3], coverage: u32) {
    let below = u32::from(pixel[3]);
    let alpha = coverage + below * (255 - coverage) / 255;
    for (channel, top) in pixel.iter_mut().zip(color) {
        let mixed =
            u32::from(top) * coverage * 255 + u32::from(*channel) * below * (255 - coverage);
        *channel = u8::try_from(mixed / (alpha * 255)).unwrap_or(u8::MAX);
    }
    pixel[3] = u8::try_from(alpha).unwrap_or(u8::MAX);
}

#[cfg(test)]
mod tests {
    use super::{IconKind, SIZE, disc_coverage, render};

    fn pixel(image: &super::Rgba, x: u32, y: u32) -> [u8; 4] {
        let at = usize::try_from((y * image.width + x) * 4).unwrap();
        image.pixels[at..at + 4].try_into().unwrap()
    }

    #[test]
    fn the_running_icon_is_the_eye_at_32_pixels() {
        let image = render(IconKind::Running).unwrap();
        assert_eq!((image.width, image.height), (SIZE, SIZE));
        assert_eq!(image.pixels.len(), 32 * 32 * 4);
        assert!(
            image.pixels.chunks_exact(4).any(|p| p[3] == 255),
            "it has solid pixels"
        );
        assert!(
            image
                .pixels
                .chunks_exact(4)
                .any(|p| p[0] != p[1] || p[1] != p[2]),
            "it has colour"
        );
    }

    #[test]
    fn the_stopped_icon_is_grey_and_faded_with_a_red_badge_at_the_bottom_right() {
        let running = render(IconKind::Running).unwrap();
        let stopped = render(IconKind::Stopped).unwrap();
        assert_eq!(
            pixel(&stopped, 24, 24),
            [220, 38, 38, 255],
            "the middle of the badge"
        );
        let rim = pixel(&stopped, 24, 17);
        assert_eq!(rim, [255, 255, 255, 255], "the white rim above the red");
        for y in 0..12 {
            for x in 0..12 {
                let [r, g, b, a] = pixel(&stopped, x, y);
                assert!(r == g && g == b, "grey outside the badge");
                assert!(
                    u32::from(a) <= u32::from(pixel(&running, x, y)[3]) * 6 / 10 + 1,
                    "faded to 60 percent"
                );
            }
        }
    }

    #[test]
    fn the_badge_edge_is_smooth_not_a_hard_cut() {
        assert_eq!(disc_coverage(0, 0, 8), 255);
        assert_eq!(disc_coverage(20, 0, 8), 0);
        let edge = disc_coverage(-8, 0, 8);
        assert!(edge > 0 && edge < 255);
    }
}
