/* @layer renderer-app @kind types */
import type { AgentGroup } from './agents-view.type';

type KeyedGroup = Exclude<AgentGroup, 'item' | 'none'>;

export type { KeyedGroup };
