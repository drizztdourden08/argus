/* @layer renderer-app @kind logic */
import type { CheckStatus, CheckStep } from '../../../../../data/remote';

const stepStatus = (step: CheckStep): CheckStatus => {
  if (step.status) return step.status;
  if (step.exitCode === null) return 'queued';
  return step.exitCode === 0 ? 'pass' : 'fail';
};

export { stepStatus };
