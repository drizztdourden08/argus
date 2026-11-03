/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import { RecordOrderContext } from './record-order-context';

const useOrderedIds = (): readonly string[] => useContext(RecordOrderContext).ids;

export { useOrderedIds };
