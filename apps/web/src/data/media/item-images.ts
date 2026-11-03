/* @layer renderer-app @kind logic */
import type { ItemImage } from '../types';

const itemImages = (item: { images?: readonly Partial<ItemImage>[] }): ItemImage[] =>
  (item.images ?? []).flatMap((image) =>
    typeof image.n === 'number' && typeof image.url === 'string' ? [{ n: image.n, url: image.url, caption: image.caption ?? '', at: image.at }] : [],
  );

export { itemImages };
