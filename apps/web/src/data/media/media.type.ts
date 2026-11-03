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

interface Shot {
  key: string;
  item: string;
  title: string;
  n: number;
  url: string;
  caption: string;
  at: string;
}

interface ShotSource {
  id: string;
  title: string;
  deletedAt?: string | null;
  images?: readonly { n: number; url: string; caption: string; at?: string }[];
}

export type { ItemImageResult, Pictures, Shot, ShotSource };
