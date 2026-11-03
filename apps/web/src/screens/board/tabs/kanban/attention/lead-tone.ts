/* @layer renderer-app @kind logic */
import type { TagUrgencyColor } from '@drizztdourden08/tessera';
import { ATTENTION_OF } from './attention.constants';
import type { AttentionKind } from './attention.type';

const leadTone = (kinds: readonly AttentionKind[]): TagUrgencyColor | undefined => (kinds.length ? ATTENTION_OF[kinds[0]].color : undefined);

export { leadTone };
