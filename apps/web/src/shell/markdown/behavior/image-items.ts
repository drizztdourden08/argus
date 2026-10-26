/* @layer renderer-app @kind logic */
import type { MarkdownImageTarget, MediaViewerItem } from '@drizztdourden08/tessera';

const imageItems = (image: MarkdownImageTarget | null): MediaViewerItem[] => (image ? [{ id: image.src, kind: 'image', src: image.src, name: image.alt || 'Image' }] : []);

export { imageItems };
