/* @layer renderer-app @kind logic */
import type { TagUrgencyColor } from '@drizztdourden08/tessera';
import { ATTENTION_OF } from './attention.constants';
import type { AttentionKind } from './attention.type';

const leadTone = (kinds: readonly AttentionKind[]): TagUrgencyColor | undefined => {
  const [lead] = kinds;
  return lead ? ATTENTION_OF[lead].color : undefined;
};

export { leadTone };
