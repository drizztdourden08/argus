/* @layer renderer-app @kind logic */
import { STAGE_LABELS } from '../builds.constants';
import type { LauncherDoc } from './model.type';

const buildStage = (launcher: LauncherDoc): string => STAGE_LABELS[launcher.retiredAt ? 'closed' : (launcher.stage ?? 'ready')] ?? 'Ready';

export { buildStage };
