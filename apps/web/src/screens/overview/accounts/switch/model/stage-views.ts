/* @layer renderer-app @kind logic */
import type { AccountsSwitch, SwitchStage, SwitchStepState } from '../../../../../data/types';
import { clockText } from '../../model';
import { NO_RECORD, NOT_REACHED, PHASE_STAGE, STAGE_LABEL, STAGE_PLAN, STAGES } from '../switch.constants';
import type { StageView } from '../switch.type';
import { isActive } from './is-active';
import { normalizeStep } from './normalize-step';

const fallbackState = (doc: AccountsSwitch, stage: SwitchStage): SwitchStepState =>
  doc.phase !== undefined && PHASE_STAGE[doc.phase] === stage ? 'running' : 'waiting';

const stageViews = (doc: AccountsSwitch): StageView[] => {
  const steps = (doc.steps ?? []).map(normalizeStep);
  const live = isActive(doc);
  return STAGES.map((stage) => {
    const last = steps.filter((step) => step.stage === stage).at(-1);
    const unrecorded = last === undefined && doc.phase === 'done';
    const state = last?.state ?? (unrecorded ? 'done' : fallbackState(doc, stage));
    const plan = unrecorded ? NO_RECORD : live || state !== 'waiting' ? STAGE_PLAN[stage] : NOT_REACHED;
    return { stage, label: STAGE_LABEL[stage], state, text: last?.text ?? plan, at: clockText(last?.at, true), detail: last?.detail ?? '' };
  });
};

export { stageViews };
