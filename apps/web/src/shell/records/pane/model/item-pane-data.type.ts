/* @layer renderer-app @kind types */
import type { RecordData } from '../pane.type';

type Shared = Pick<RecordData, 'boardKey' | 'record' | 'now' | 'ops' | 'acts' | 'feed' | 'open'>;

export type { Shared };
