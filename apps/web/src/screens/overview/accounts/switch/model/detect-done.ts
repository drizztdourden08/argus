/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';

const detectDone = (doc: AccountsSwitch): boolean =>
  (doc.threads ?? []).length > 0 || (doc.steps ?? []).some((step) => step.stage === 'detect' && step.state !== 'running' && step.state !== 'waiting');

export { detectDone };
