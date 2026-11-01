//! The tray icons as pixels: the Argus mark, white on a dark taskbar and black on a light one,
//! and the same mark greyed and faded with a red badge for every other state. Pure, so it is
//! tested without a window.
// test-tag: critical

use std::io::Cursor;

use ico::IconImage;

use super::error::TrayError;

/// The icon file of Argus, for the identity Windows shows on a toast.
pub const ARGUS_ICO: &[u8] = include_bytes!("../../assets/argus.ico");
const SAMPLES: i64 = 4;
const RED: [u8; 3] = [220, 38, 38];
const WHITE: [u8; 3] = [255, 255, 255];

/// The mark in each ink at the sizes the taskbar asks for, smallest first.
const MARKS: [(Ink, u32, &[u8]); 8] = [
    (
        Ink::White,
        16,
        include_bytes!("../../assets/argus-white-16.png"),
    ),
    (
        Ink::White,
        24,
        include_bytes!("../../assets/argus-white-24.png"),
    ),
    (
        Ink::White,
        32,
        include_bytes!("../../assets/argus-white-32.png"),
    ),
    (
        Ink::White,
        48,
        include_bytes!("../../assets/argus-white-48.png"),
    ),
    (
        Ink::Black,
        16,
        include_bytes!("../../assets/argus-black-16.png"),
    ),
    (
        Ink::Black,
        24,
        include_bytes!("../../assets/argus-black-24.png"),
    ),
    (
        Ink::Black,
        32,
        include_bytes!("../../assets/argus-black-32.png"),
    ),
    (
        Ink::Black,
        48,
        include_bytes!("../../assets/argus-black-48.png"),
    ),
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

/// The icon of Argus for a state and an ink, from the smallest embedded size of at least
/// `wanted` pixels (the largest when none is that big).
///
/// # Errors
///
/// `Platform` when an embedded image is unusable.
pub fn render(kind: IconKind, ink: Ink, wanted: u32) -> Result<Rgba, TrayError> {
    let base = decode(ink, wanted)?;
    Ok(match kind {
        IconKind::Running => base,
        IconKind::Stopped => stopped(&base),
    })
}

fn decode(ink: Ink, wanted: u32) -> Result<Rgba, TrayError> {
    let fail = |why: String| TrayError::Platform(format!("the Argus icon is unusable: {why}"));
    let marks: Vec<_> = MARKS.iter().filter(|(own, ..)| *own == ink).collect();
    let (_, _, bytes) = marks
        .iter()
        .find(|(_, size, _)| *size >= wanted)
        .or(marks.last())
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

    use super::{ARGUS_ICO, IconKind, Ink, Rgba, render};

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
        colours.dedup();
        colours
    }

    #[test]
    fn the_running_icon_is_the_mark_at_the_size_asked() {
        for wanted in [16, 24, 32, 48] {
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
    fn a_size_between_two_takes_the_next_one_up_and_a_huge_one_the_largest() {
        assert_eq!(render(IconKind::Running, Ink::White, 20).unwrap().width, 24);
        assert_eq!(
            render(IconKind::Running, Ink::White, 100).unwrap().width,
            48
        );
    }

    #[test]
    fn the_mark_is_white_on_a_dark_taskbar_and_black_on_a_light_one() {
        let white = render(IconKind::Running, Ink::White, 32).unwrap();
        let black = render(IconKind::Running, Ink::Black, 32).unwrap();
        assert_eq!(solid_colours(&white), vec![[255, 255, 255]]);
        assert_eq!(solid_colours(&black), vec![[0, 0, 0]]);
    }

    #[test]
    fn the_identity_icon_file_holds_every_size() {
        let dir = IconDir::read(std::io::Cursor::new(ARGUS_ICO)).unwrap();
        let mut sizes: Vec<u32> = dir.entries().iter().map(ico::IconDirEntry::width).collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![16, 24, 32, 48, 256]);
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
