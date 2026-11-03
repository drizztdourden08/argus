/* @layer renderer-app @kind logic */
import type { ItemImage } from '../types';

const mergeImages = (kept: readonly ItemImage[], added: readonly ItemImage[]): ItemImage[] => {
  const known = new Set(kept.map((image) => image.n));
  return [...kept, ...added.filter((image) => !known.has(image.n))].sort((a, b) => a.n - b.n);
};

export { mergeImages };
