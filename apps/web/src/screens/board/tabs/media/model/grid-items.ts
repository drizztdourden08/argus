/* @layer renderer-app @kind logic */
import type { MediaGridItem } from '@drizztdourden08/tessera';
import type { Shot } from '../../../../../data/media';

const gridItems = (shots: readonly Shot[]): MediaGridItem[] =>
  shots.map((shot) => ({ src: shot.url, caption: shot.caption || `Image ${shot.n}`, source: shot.item }));

export { gridItems };
