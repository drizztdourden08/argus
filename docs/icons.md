<!-- @layer docs @kind doc -->
# Native icons

The Argus mark shows in the app through Tessera's `BrandMark app="argus"`, drawn in `currentColor`.
The file `apps/web/assets/argus-logo.svg` is the same mark for the tray icon script, a single colour
drawn with `currentColor`. The owner's rule: the mark is white or black depending on where it sits, or the
palette primary when it needs emphasis.

## Where it shows

- **The tray.** White on a dark taskbar and black on a light one. The tray reads
  `SystemUsesLightTheme` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`
  at start and every half second, and draws again when it changes. The size follows the small
  icon metric of Windows for the DPI of the screen (`GetSystemMetricsForDpi(SM_CXSMICON)`): 16, 20,
  24, 28, 32, 40, 48 or 64. A stopped or starting server is grey and faded with
  a red badge at the bottom right.
- **Toasts and the app identity.** `argus.ico` in the basic Tessera primary, so it reads on a
  light or a dark toast. The server writes it to the Argus home and registers it as the `IconUri`
  of `Argus.Dashboard`. Toasts carry no image of their own.

## DPI

The tray process declares itself per-monitor DPI aware (version 2) before it creates its window.
Without that, Windows reports a 16 pixel slot on a 125 or 150 percent screen and then stretches
the 16 pixel bitmap to 20 or 24 pixels, which is the blur. The tray reads the slot size for the
DPI of its own window and looks again every half second with the theme, so moving to a screen of
another scale, or changing the scale, draws the icon again. No bitmap is ever scaled up: the exact
size is drawn, and a size with no drawing (36 pixels at 225 percent) takes the next larger
drawing shrunk once by an area average.

## How the files are made

`pnpm icons` runs `tools/icons/render-argus-icons.mjs`. It writes into `crates/argus-native/assets`:

- `argus-white-N.png` and `argus-black-N.png` for N in 16, 20, 24, 28, 32, 40, 48 and 64 (the
  sizes of a small icon at 100, 125, 150, 175, 200, 250, 300 and 400 percent), and 256.
- `argus.ico` with the sizes 16, 20, 24, 32, 40, 48 and 256 as PNG images.

The mark is fitted to its own bounding box, not to a square: the painted width fills the square
minus a margin, and the top edge falls on a pixel boundary, so the horizontal edges sit between
two rows of pixels. At 32 pixels and below the script adds a stroke of 0.3 pixel in the same
colour, so the gaps of the eye and the A do not close up. At 16 pixels the edges are not
anti-aliased (`shape-rendering="crispEdges"`): every pixel is fully on or off, so nothing is
soft. Run it again after any change to the SVG, and commit the outputs.
