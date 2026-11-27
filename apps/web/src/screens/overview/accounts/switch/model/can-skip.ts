/* @layer renderer-app @kind logic */
import type { AccountsSwitch, SwitchStage } from '../../../../../data/types';
import { PHASE_STAGE, SKIPPABLE_STAGES } from '../switch.constants';

const canSkip = (doc: AccountsSwitch, stage: SwitchStage | undefined): stage is SwitchStage =>
  stage !== undefined && SKIPPABLE_STAGES.has(stage) && doc.phase !== undefined && PHASE_STAGE[doc.phase] === stage;

export { canSkip };
