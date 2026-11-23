/* @layer renderer-app @kind logic */
import type { AccountsSwitch, SwitchStage, SwitchStepState } from '../../../../../data/types';
import { clockText } from '../../model';
import { NOT_REACHED, PHASE_STAGE, STAGE_LABEL, STAGE_PLAN, STAGES } from '../switch.constants';
import type { StageView } from '../switch.type';
import { isActive } from './is-active';

const fallbackState = (doc: AccountsSwitch, stage: SwitchStage): SwitchStepState =>
  doc.phase !== undefined && PHASE_STAGE[doc.phase] === stage ? 'running' : 'waiting';

const stageViews = (doc: AccountsSwitch): StageView[] => {
  const steps = doc.steps ?? [];
  const live = isActive(doc);
  return STAGES.map((stage) => {
    const last = steps.filter((step) => step.stage === stage).at(-1);
    const state = last?.state ?? fallbackState(doc, stage);
    const plan = live || state !== 'waiting' ? STAGE_PLAN[stage] : NOT_REACHED;
    return { stage, label: STAGE_LABEL[stage], state, text: last?.text ?? plan, at: clockText(last?.at, true), detail: last?.detail ?? '' };
  });
};

export { stageViews };
