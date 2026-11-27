/* @layer renderer-app @kind logic */
import type { AccountsSwitch, SwitchStage } from '../../../../../data/types';
import { STAGES } from '../switch.constants';
import { normalizeStep } from './normalize-step';

const wizardOrder = (doc: AccountsSwitch): SwitchStage[] => {
  const seen = (doc.steps ?? []).map(normalizeStep).flatMap((step) => step.stage ?? []);
  return [...new Set([...seen, ...STAGES])];
};

export { wizardOrder };
