/* @layer renderer-app @kind logic */
import type { SwitchStage } from '../../../../../data/types';
import type { StageView } from '../switch.type';

const currentStage = (views: readonly StageView[]): SwitchStage | undefined =>
  (views.find((view) => view.state === 'running') ?? views.find((view) => view.state === 'waiting') ?? views.at(-1))?.stage;

export { currentStage };
