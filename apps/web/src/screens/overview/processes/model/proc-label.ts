/* @layer renderer-app @kind logic */
import type { ProcDoc } from '../../../../data/types';

const procLabel = (proc: ProcDoc): string => `${proc.name ? String(proc.name) : 'process'} ${proc.pid ?? '?'}`;

export { procLabel };
