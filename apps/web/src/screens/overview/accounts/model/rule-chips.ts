/* @layer renderer-app @kind logic */
import type { AccountRules } from '../../../../data/types';
import { DEFAULT_SWITCH_AT } from '../accounts.constants';

const handoverText = (handover: AccountRules['handover']): string => {
  if (typeof handover === 'number') return `Handover at ${handover}%`;
  return handover === 'auto' ? 'Handover auto' : 'Handover off';
};

const ruleChips = (rules: AccountRules | undefined): string[] => [
  rules?.autoSwitch ? `Auto-switch at ${rules.switchAt ?? DEFAULT_SWITCH_AT}%` : 'Auto-switch off',
  handoverText(rules?.handover),
];

export { ruleChips };
