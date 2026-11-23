/* @layer renderer-app @kind logic */
import type { AccountsSwitch, SwitchStage, SwitchStepDoc, SwitchStepState } from '../../../../../data/types';
import { clockText } from '../../model';
import { NO_RECORD, NOT_REACHED, PHASE_STAGE, STAGE_LABEL, STAGE_PLAN, STAGES } from '../switch.constants';
import type { StageView } from '../switch.type';
import { isActive } from './is-active';
import { normalizeStep } from './normalize-step';

const fallbackState = (doc: AccountsSwitch, stage: SwitchStage): SwitchStepState =>
  doc.phase !== undefined && PHASE_STAGE[doc.phase] === stage ? 'running' : 'waiting';

const planText = (stage: SwitchStage, state: SwitchStepState, live: boolean): string =>
  live || state !== 'waiting' ? STAGE_PLAN[stage] : NOT_REACHED;

const stateOf = (doc: AccountsSwitch, last: SwitchStepDoc | undefined, stage: SwitchStage, unrecorded: boolean): SwitchStepState =>
  last?.state ?? (unrecorded ? 'done' : fallbackState(doc, stage));

const textOf = (doc: AccountsSwitch, last: SwitchStepDoc | undefined, stage: SwitchStage, unrecorded: boolean): string => {
  const own = last?.text ?? undefined;
  if (own !== undefined) return own;
  return unrecorded ? NO_RECORD : planText(stage, stateOf(doc, last, stage, unrecorded), isActive(doc));
};

const stageView = (doc: AccountsSwitch, steps: SwitchStepDoc[], stage: SwitchStage): StageView => {
  const last = steps.filter((step) => step.stage === stage).at(-1);
  const unrecorded = last === undefined && doc.phase === 'done';
  const state = stateOf(doc, last, stage, unrecorded);
  return { stage, label: STAGE_LABEL[stage], state, text: textOf(doc, last, stage, unrecorded), at: clockText(last?.at, true), detail: last?.detail ?? '' };
};

const stageViews = (doc: AccountsSwitch): StageView[] => {
  const steps = (doc.steps ?? []).map(normalizeStep);
  return STAGES.map((stage) => stageView(doc, steps, stage));
};

export { stageViews };
