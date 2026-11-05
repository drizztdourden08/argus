/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { FALLBACK_COLORS, PALETTE_ATTRIBUTE, TRANSITION_MS } from './backdrop.constants';
import { mix } from './mix';
import { toHex } from './to-hex';
import type { Rgb } from './color-math.type';
import { readPaletteColors } from './palette-colors';

const useBackdropColors = (): readonly string[] => {
  const [colors, setColors] = useState<readonly string[]>(FALLBACK_COLORS);

  useEffect(() => {
    let shown: readonly Rgb[] = [];
    let frame = 0;
    const transitionTo = (target: readonly Rgb[]): void => {
      cancelAnimationFrame(frame);
      const from = shown.length > 0 ? shown : target;
      const start = performance.now();
      const step = (now: number): void => {
        const t = Math.min(1, (now - start) / TRANSITION_MS);
        shown = target.map((color, index) => mix(from[index] ?? color, color, t));
        setColors(shown.map(toHex));
        if (t < 1) frame = requestAnimationFrame(step);
      };
      frame = requestAnimationFrame(step);
    };
    const refresh = (): void => {
      const next = readPaletteColors();
      if (next.length > 0) transitionTo(next);
    };
    refresh();
    const observer = new MutationObserver(refresh);
    observer.observe(document.documentElement, { attributes: true, attributeFilter: [PALETTE_ATTRIBUTE] });
    return () => {
      observer.disconnect();
      cancelAnimationFrame(frame);
    };
  }, []);

  return colors;
};

export { useBackdropColors };
