//! The tray icons as pixels: the Argus mark, white on a dark taskbar and black on a light one,
//! and the same mark greyed and faded with a red badge for every other state. Pure, so it is
//! tested without a window.
// test-tag: critical

use std::io::Cursor;

use ico::IconImage;

use super::downscale::downscale;
use super::error::TrayError;

/// The icon file of Argus, for the identity Windows shows on a toast.
pub const ARGUS_ICO: &[u8] = include_bytes!("../../assets/argus.ico");
const SAMPLES: i64 = 4;
const RED: [u8; 3] = [220, 38, 38];
const WHITE: [u8; 3] = [255, 255, 255];

/// One embedded picture of the mark: its ink, its size and its PNG bytes.
macro_rules! mark {
    ($ink:ident, $name:literal, $size:literal) => {
        (
            Ink::$ink,
            $size,
            include_bytes!(concat!("../../assets/argus-", $name, "-", $size, ".png")),
        )
    };
}

/// The mark in each ink at every size Windows gives a small icon (96 to 384 dpi: 16, 20, 24, 28,
/// 32, 40, 48, 64), smallest first. Each is drawn for its own size, so the exact one is shown
/// as it is.
const MARKS: [(Ink, u32, &[u8]); 16] = [
    mark!(White, "white", 16),
    mark!(White, "white", 20),
    mark!(White, "white", 24),
    mark!(White, "white", 28),
    mark!(White, "white", 32),
    mark!(White, "white", 40),
    mark!(White, "white", 48),
    mark!(White, "white", 64),
    mark!(Black, "black", 16),
    mark!(Black, "black", 20),
    mark!(Black, "black", 24),
    mark!(Black, "black", 28),
    mark!(Black, "black", 32),
    mark!(Black, "black", 40),
    mark!(Black, "black", 48),
    mark!(Black, "black", 64),
];

/// Which of the two icons to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    /// The server runs: the icon as it is.
    Running,
    /// The server is down or not known yet: grey, faded, with a red badge.
    Stopped,
}

/// The colour of the mark: the one that stands out on the taskbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// On a dark taskbar.
    White,
    /// On a light taskbar.
    Black,
}

impl Ink {
    /// The ink for the `SystemUsesLightTheme` value of Windows: 0 is a dark taskbar, anything
    /// else a light one, and a missing value is taken as dark.
    #[must_use]
    pub const fn for_light_theme(value: Option<u32>) -> Self {
        match value {
            Some(0) | None => Self::White,
            Some(_) => Self::Black,
        }
    }
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

/// The icon of Argus for a state and an ink at `wanted` pixels: the drawing made for exactly
/// that size when there is one, else the next larger one shrunk once. Never enlarged, so a size
/// beyond the largest drawing gets the largest as it is.
///
/// # Errors
///
/// `Platform` when an embedded image is unusable.
pub fn render(kind: IconKind, ink: Ink, wanted: u32) -> Result<Rgba, TrayError> {
    let base = downscale(&decode(ink, wanted)?, wanted);
    Ok(match kind {
        IconKind::Running => base,
        IconKind::Stopped => stopped(&base),
    })
}

/// Which of the drawn `sizes` (smallest first) serves `wanted`: the exact one, else the smallest
/// that is larger, else the largest when none is.
fn pick(sizes: &[u32], wanted: u32) -> Option<u32> {
    sizes
        .iter()
        .copied()
        .find(|size| *size >= wanted)
        .or_else(|| sizes.last().copied())
}

fn decode(ink: Ink, wanted: u32) -> Result<Rgba, TrayError> {
    let fail = |why: String| TrayError::Platform(format!("the Argus icon is unusable: {why}"));
    let marks: Vec<_> = MARKS.iter().filter(|(own, ..)| *own == ink).collect();
    let sizes: Vec<u32> = marks.iter().map(|(_, size, _)| *size).collect();
    let (_, _, bytes) = pick(&sizes, wanted)
        .and_then(|chosen| marks.iter().find(|(_, size, _)| *size == chosen))
        .ok_or_else(|| fail("no image".to_owned()))?;
    let image = IconImage::read_png(Cursor::new(bytes)).map_err(|e| fail(e.to_string()))?;
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
    use ico::IconDir;

    use super::{ARGUS_ICO, IconKind, Ink, MARKS, Rgba, pick, render};

    fn pixel(image: &Rgba, x: u32, y: u32) -> [u8; 4] {
        let at = usize::try_from((y * image.width + x) * 4).unwrap();
        image.pixels[at..at + 4].try_into().unwrap()
    }

    fn solid_colours(image: &Rgba) -> Vec<[u8; 3]> {
        let mut colours: Vec<[u8; 3]> = image
            .pixels
            .chunks_exact(4)
            .filter(|p| p[3] == 255)
            .map(|p| [p[0], p[1], p[2]])
            .collect();
        colours.sort_unstable();
        colours.dedup();
        colours
    }

    const EMERALD: [u8; 3] = [18, 160, 110];

    fn is_stone(colour: [u8; 3]) -> bool {
        let (low, high) = (colour.iter().min().unwrap(), colour.iter().max().unwrap());
        high - low <= 10 && *low > 0 && *high < 255
    }

    #[test]
    fn the_running_icon_is_the_mark_at_the_size_asked() {
        for wanted in [16, 20, 24, 28, 32, 40, 48, 64] {
            let image = render(IconKind::Running, Ink::White, wanted).unwrap();
            assert_eq!((image.width, image.height), (wanted, wanted));
            assert_eq!(image.pixels.len(), (wanted * wanted * 4) as usize);
            assert!(
                image.pixels.chunks_exact(4).any(|p| p[3] == 255),
                "it has solid pixels"
            );
            assert!(
                image.pixels.chunks_exact(4).any(|p| p[3] == 0),
                "it has clear pixels around the mark"
            );
        }
    }

    #[test]
    fn a_size_between_two_is_the_next_one_up_shrunk_once_and_a_huge_one_the_largest() {
        let between = render(IconKind::Running, Ink::White, 36).unwrap();
        assert_eq!((between.width, between.height), (36, 36));
        assert!(
            between.pixels.chunks_exact(4).any(|p| p[3] == 255),
            "it keeps solid pixels"
        );
        let huge = render(IconKind::Running, Ink::White, 100).unwrap();
        assert_eq!((huge.width, huge.height), (64, 64));
    }

    #[test]
    fn every_drawn_size_is_shown_as_it_is_without_any_scaling() {
        for (ink, size, bytes) in MARKS {
            let drawn = ico::IconImage::read_png(std::io::Cursor::new(bytes)).unwrap();
            let shown = render(IconKind::Running, ink, size).unwrap();
            assert_eq!((drawn.width(), drawn.height()), (size, size));
            assert_eq!(shown.pixels, drawn.rgba_data());
        }
    }

    #[test]
    fn the_size_is_the_exact_one_else_the_next_larger_and_never_a_smaller() {
        let sizes = [16, 20, 24, 28, 32, 40, 48, 64];
        for exact in sizes {
            assert_eq!(pick(&sizes, exact), Some(exact));
        }
        assert_eq!(pick(&sizes, 36), Some(40));
        assert_eq!(pick(&sizes, 17), Some(20));
        assert_eq!(pick(&sizes, 8), Some(16));
        assert_eq!(pick(&sizes, 65), Some(64), "none larger: the largest");
        assert_eq!(pick(&[], 16), None);
    }

    #[test]
    fn the_mark_is_the_same_colour_picture_on_a_dark_and_a_light_taskbar() {
        for wanted in [16, 32, 64] {
            let dark = render(IconKind::Running, Ink::White, wanted).unwrap();
            let light = render(IconKind::Running, Ink::Black, wanted).unwrap();
            assert_eq!(dark, light, "both inks hold the same mark at {wanted}");
            let colours = solid_colours(&dark);
            assert!(colours.contains(&EMERALD), "the emerald facet is there");
            assert!(
                colours.iter().all(|c| *c == EMERALD || is_stone(*c)),
                "only the stone greys and the emerald, no pure white or black: {colours:?}"
            );
        }
    }

    #[test]
    fn the_identity_icon_file_holds_every_size() {
        let dir = IconDir::read(std::io::Cursor::new(ARGUS_ICO)).unwrap();
        let mut sizes: Vec<u32> = dir.entries().iter().map(ico::IconDirEntry::width).collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![16, 20, 24, 32, 40, 48, 256]);
        for entry in dir.entries() {
            entry.decode().unwrap();
        }
    }

    #[test]
    fn the_taskbar_theme_picks_the_ink() {
        assert_eq!(Ink::for_light_theme(Some(0)), Ink::White);
        assert_eq!(Ink::for_light_theme(Some(1)), Ink::Black);
        assert_eq!(Ink::for_light_theme(None), Ink::White);
    }

    #[test]
    fn the_stopped_icon_is_grey_and_faded_with_a_red_badge_at_the_bottom_right() {
        let running = render(IconKind::Running, Ink::White, 32).unwrap();
        let stopped = render(IconKind::Stopped, Ink::White, 32).unwrap();
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
}
