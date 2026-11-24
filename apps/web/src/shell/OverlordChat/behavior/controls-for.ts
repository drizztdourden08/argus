/* @layer renderer-app @kind logic */
import type { OverlordState } from '../../../data/overlord';
import { CONTROLS_BY_STATE, NO_CONTROLS } from './panel-model.constants';
import type { ControlSet } from './panel-model.type';

const controlsFor = (state: OverlordState | undefined): ControlSet => (state ? CONTROLS_BY_STATE[state] : NO_CONTROLS);

export { controlsFor };
