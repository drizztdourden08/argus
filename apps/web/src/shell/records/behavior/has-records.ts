/* @layer renderer-app @kind logic */
import { splitRecords } from './split-records';

const hasRecords = (text: string): boolean => splitRecords(text).some((part) => part.ref !== null);

export { hasRecords };
