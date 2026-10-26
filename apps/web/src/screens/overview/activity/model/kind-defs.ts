/* @layer renderer-app @kind logic */
import type { LogKindDef } from '@drizztdourden08/tessera';
import { kindOf } from './kind-of';
import { KIND_FALLBACK_TONE, KIND_TONE, KINDS } from './model.constants';
import type { ActivityItem } from './model.type';

const kindDefs = (items: readonly ActivityItem[]): LogKindDef[] => {
  const ids = [...new Set<string>([...KINDS, ...items.map(kindOf)])];
  return ids.map((id) => ({ id, label: id, tone: KIND_TONE[id] ?? KIND_FALLBACK_TONE }));
};

export { kindDefs };
