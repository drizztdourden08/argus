/* @layer renderer-app @kind logic */
import { BASE_TOKEN, COLOR_TOKENS, DARKEN_AMOUNT } from './backdrop.constants';
import { darken } from './darken';
import type { Rgb } from './color-math.type';

const parseCssColor = (css: string): Rgb | undefined => {
  const context = document.createElement('canvas').getContext('2d', { willReadFrequently: true });
  if (!context || css === '') return undefined;
  context.clearRect(0, 0, 1, 1);
  context.fillStyle = css;
  context.fillRect(0, 0, 1, 1);
  const [red, green, blue, alpha] = context.getImageData(0, 0, 1, 1).data;
  return alpha === 0 ? undefined : [red, green, blue];
};

const readPaletteColors = (): readonly Rgb[] => {
  const styles = getComputedStyle(document.documentElement);
  const colors: Rgb[] = [];
  for (const token of COLOR_TOKENS) {
    const parsed = parseCssColor(styles.getPropertyValue(token).trim());
    if (parsed) colors.push(token === BASE_TOKEN ? parsed : darken(parsed, DARKEN_AMOUNT));
  }
  return colors;
};

export { readPaletteColors };
