/* @layer renderer-app @kind logic */
import type { OverlordMode } from '../../../data/overlord';
import { MODE_LABELS, MODE_REFUSALS, MODE_STEPS, MODES } from './panel-model.constants';
import type { ModeStep } from './panel-model.type';

const stepFor = (current: OverlordMode, mode: OverlordMode): ModeStep => {
  const isCurrent = mode === current;
  const allowed = isCurrent || MODE_STEPS[current].includes(mode);
  return { mode, label: MODE_LABELS[mode], current: isCurrent, allowed, reason: allowed ? null : MODE_REFUSALS[mode] };
};

const modeSteps = (current: OverlordMode): ModeStep[] => MODES.map((mode) => stepFor(current, mode));

export { modeSteps };
