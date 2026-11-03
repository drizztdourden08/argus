/* @layer renderer-app @kind logic */
import { createContext } from 'react';
import type { RecordOrder } from '../records.type';

const NO_IDS: readonly string[] = [];

const nobody = (): void => undefined;

const RecordOrderContext = createContext<RecordOrder>({ ids: NO_IDS, set: nobody });

export { RecordOrderContext };
