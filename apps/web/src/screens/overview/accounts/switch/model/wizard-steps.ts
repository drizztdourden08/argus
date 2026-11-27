/* @layer renderer-app @kind logic */
import type { StepperStep } from '@drizztdourden08/tessera';
import type { AccountsSwitch } from '../../../../../data/types';
import { STEPPER_STATE } from '../switch.constants';
import type { StageView, WizardModel } from '../switch.type';
import { currentStage } from './current-stage';
import { stageViews } from './stage-views';
import { stepSummary } from './step-summary';
import { wizardOrder } from './wizard-order';

const stepOf = (doc: AccountsSwitch, view: StageView): StepperStep => ({
  id: view.stage,
  label: view.label,
  summary: stepSummary(doc, view),
  state: STEPPER_STATE[view.state],
  error: view.state === 'failed',
  struck: view.state === 'skipped',
});

const wizardSteps = (doc: AccountsSwitch): WizardModel => {
  const byStage = new Map(stageViews(doc).map((view) => [view.stage, view]));
  const views = wizardOrder(doc).flatMap((stage) => byStage.get(stage) ?? []);
  return { steps: views.map((view) => stepOf(doc, view)), currentId: currentStage(views), complete: doc.phase === 'done' };
};

export { wizardSteps };
