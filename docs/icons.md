<!-- @layer docs @kind doc -->
# Native icons

The Argus mark is a faceted stone shield with two eye holes and one emerald facet at the top right
corner (logo 8, row A). The eyes are real cutouts: the background shows through. Row B is the same
facets in emerald tones, for use in Tessera. The mark is no longer `currentColor`: it carries its own colour,
so the tray uses the same picture on a light and a dark taskbar.

## The logo geometry

`tools/icons/argus-facets.mjs` defines the shield once: the outline, the straight-edged facets that
share their vertices exactly, the two eye squares, which facet is the accent, and the tones of A and B.
It holds three detail levels of the same outline, each on a whole-number grid:

| Level | Grid | Facets | Sizes in pixels |
|---|---|---|---|
| `full` | 64 | 9 | 64 and up |
| `medium` | 32 | 6 | 32 to 48 |
| `small` | 16 | 3 | 16 to 28 |

`tools/icons/generate-argus-logos.mjs` writes `apps/web/assets/logo/argus-{a,b}-{full,medium,small}.svg`:
flat colours, no gradients, no strokes. Each facet that holds an eye is one path with the eye square
subtracted (`fill-rule="evenodd"`). The `small` files use `shape-rendering="crispEdges"`. For a raster
size that is not the grid of its level (20, 24 and 28 from `small`, 40 and 48 from `medium`), the
generator snaps every vertex to whole pixels, mirrored about the centre line so the shield stays
symmetric, and the eyes land on whole pixels too. It throws if a snapped eye leaves its facet.

## Where it shows

- **The tray.** The colour mark on both taskbars (the files `argus-white-N.png` and `argus-black-N.png`
  are the same picture, kept so the tray still picks by theme). The tray reads
  `SystemUsesLightTheme` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`
  at start and every half second, and draws again when it changes. The size follows the small
  icon metric of Windows for the DPI of the screen (`GetSystemMetricsForDpi(SM_CXSMICON)`): 16, 20,
  24, 28, 32, 40, 48 or 64. A stopped or starting server is grey and faded with
  a red badge at the bottom right.
- **Toasts and the app identity.** `argus.ico`, the colour mark, so it reads on a
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

`pnpm icons` runs `tools/icons/render-argus-icons.mjs`. It first writes the six SVGs, then writes into
`crates/argus-native/assets`:

- `argus-white-N.png` and `argus-black-N.png` for N in 16, 20, 24, 28, 32, 40, 48 and 64 (the
  sizes of a small icon at 100, 125, 150, 175, 200, 250, 300 and 400 percent), and 256. Both name
  sets hold the same A mark: the `small` level for 16 to 28, `medium` for 32 to 48, `full` for 64 and 256.
- `argus.ico` with the sizes 16, 20, 24, 32, 40, 48 and 256 as PNG images, from the same levels.

Every size is drawn from a snapped copy of its level, never scaled from a larger picture. Run it again
after any change to the facets, and commit the outputs.
