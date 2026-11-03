/* @layer renderer-app @kind logic */
import type { RunStatus } from '../../../../../../../data/kanban';
import { RUN_OUTCOMES } from '../run.constants';

const isRunStatus = (value: unknown): value is RunStatus => RUN_OUTCOMES.some((outcome) => outcome.status === value);

export { isRunStatus };
