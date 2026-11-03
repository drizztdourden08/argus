/* @layer renderer-app @kind logic */
import { createContext } from 'react';
import type { RecordOrder } from '../records.type';
import { NO_IDS } from './record-order-context.constants';

const nobody = (): void => undefined;

const RecordOrderContext = createContext<RecordOrder>({ ids: NO_IDS, set: nobody });

export { RecordOrderContext };
