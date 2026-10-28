<!-- @layer docs @kind doc -->
# Native icons

The Argus mark lives in `apps/web/assets/argus-logo.svg`, a single colour drawn with
`currentColor`. The owner's rule: the mark is white or black depending on where it sits, or the
palette primary when it needs emphasis.

## Where it shows

- **The tray.** White on a dark taskbar and black on a light one. The tray reads
  `SystemUsesLightTheme` under `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`
  at start and every half second, and draws again when it changes. The size follows the small
  icon metric of Windows (16, 24, 32 or 48). A stopped or starting server is grey and faded with
  a red badge at the bottom right.
- **Toasts and the app identity.** `argus.ico` in the basic Tessera primary, so it reads on a
  light or a dark toast. The server writes it to the Argus home and registers it as the `IconUri`
  of `Argus.Dashboard`. Toasts carry no image of their own.

## How the files are made

`pnpm icons` runs `tools/icons/render-argus-icons.mjs`. It writes into `crates/argus-native/assets`:

- `argus-white-N.png` and `argus-black-N.png` for N in 16, 24, 32, 48 and 256.
- `argus.ico` with the same five sizes as PNG images.

The mark is cropped to a square around its centre. At 32 pixels and below the script adds a
stroke in the same colour (8, 14 and 20 units of the 1150 unit box for 32, 24 and 16 pixels), so
the gaps of the eye and the A do not close up. Run it again after any change to the SVG, and
commit the outputs.
