/* @layer renderer-app @kind logic */
import type { ItemImage } from '../types';
import { IMAGE_REF } from './media.constants';
import type { Pictures } from './media.type';

const picturesOf = (images: readonly ItemImage[]): Pictures => ({
  resolve: (src) => {
    const n = IMAGE_REF.exec(src)?.[1];
    return n === undefined ? undefined : images.find((image) => image.n === Number(n))?.url;
  },
  gallery: images.map((image) => ({ id: String(image.n), kind: 'image' as const, src: image.url, name: image.caption || `Image ${image.n}` })),
});

export { picturesOf };
