/* @layer renderer-app @kind logic */
import { RUN_CLOSED_STAGES } from './model.constants';

const runClosed = (stage: string | null | undefined): boolean => RUN_CLOSED_STAGES.includes(stage ?? '');

export { runClosed };
