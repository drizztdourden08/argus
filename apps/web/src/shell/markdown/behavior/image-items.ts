/* @layer renderer-app @kind logic */
import type { MarkdownImageTarget, MediaViewerItem } from '@drizztdourden08/tessera';

const imageItems = (image: MarkdownImageTarget | null, gallery: readonly MediaViewerItem[] = []): readonly MediaViewerItem[] => {
  if (!image) return [];
  if (gallery.some((item) => item.src === image.src)) return gallery;
  return [{ id: image.src, kind: 'image', src: image.src, name: image.alt || 'Image' }];
};

export { imageItems };
