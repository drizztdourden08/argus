/* @layer renderer-app @kind types */
import type { MediaViewerItem } from '@drizztdourden08/tessera';

interface ItemImageResult {
  item: string;
  image: { n: number; file: string; url: string; caption: string };
}

interface Pictures {
  resolve: (src: string) => string | undefined;
  gallery: readonly MediaViewerItem[];
}

export type { ItemImageResult, Pictures };
