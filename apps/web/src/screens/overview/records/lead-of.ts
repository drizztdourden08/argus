/* @layer renderer-app @kind logic */
import { LEAD_PREFIX } from './records.constants';

const leadOf = (project: string | null | undefined): string => (project ? `${LEAD_PREFIX}${project}` : '');

export { leadOf };
