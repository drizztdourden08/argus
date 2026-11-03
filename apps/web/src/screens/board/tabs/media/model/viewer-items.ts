/* @layer renderer-app @kind logic */
import type { MediaViewerItem } from '@drizztdourden08/tessera';
import type { Shot } from '../../../../../data/media';

const viewerItems = (shots: readonly Shot[]): MediaViewerItem[] =>
  shots.map((shot) => ({ id: shot.key, kind: 'image', src: shot.url, name: `${shot.item}: ${shot.caption || `Image ${shot.n}`}` }));

export { viewerItems };
