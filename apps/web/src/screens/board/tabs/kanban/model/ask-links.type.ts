/* @layer renderer-app @kind types */
import type { Entry } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { AskKind } from './model.type';

interface Asking {
  entry: WithId<Entry>;
  kind: AskKind;
}

export type { Asking };
