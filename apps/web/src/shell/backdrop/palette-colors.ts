/* @layer renderer-app @kind logic */
import { COLOR_TOKENS, DARKEN_AMOUNT, DEEP_LIFT, DEEP_TOKENS } from './backdrop.constants';
import { darken } from './darken';
import { mix } from './mix';
import type { Rgb } from './color-math.type';

const parseCssColor = (css: string): Rgb | undefined => {
  const context = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
  if (!context || css === '') return undefined;
  context.clearRect(0, 0, 1, 1);
  context.fillStyle = css;
  context.fillRect(0, 0, 1, 1);
  const [red = 0, green = 0, blue = 0, alpha = 0] = context.getImageData(0, 0, 1, 1).data;
  return alpha === 0 ? undefined : [red, green, blue];
};

/* The deep tones are lifted toward the first spot, so they read as shades of the palette and not as black holes. */
const toSpot = (token: string, color: Rgb, lead: Rgb | undefined): Rgb => {
  if (!DEEP_TOKENS.includes(token)) return darken(color, DARKEN_AMOUNT);
  return lead ? mix(color, lead, DEEP_LIFT) : color;
};

const readPaletteColors = (): readonly Rgb[] => {
  const styles = getComputedStyle(document.documentElement);
  const colors: Rgb[] = [];
  for (const token of COLOR_TOKENS) {
    const parsed = parseCssColor(styles.getPropertyValue(token).trim());
    if (parsed) colors.push(toSpot(token, parsed, colors[0]));
  }
  return colors;
};

export { readPaletteColors };
