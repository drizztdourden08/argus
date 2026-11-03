/* @layer renderer-app @kind logic */
import type { RunStatus } from '../../../../../../../data/kanban';
import { RUN_OUTCOMES } from '../run.constants';

const keyStatus = (key: string): RunStatus | null => RUN_OUTCOMES.find((outcome) => outcome.key === key.toLowerCase())?.status ?? null;

export { keyStatus };
