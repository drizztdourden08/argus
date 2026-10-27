/* @layer renderer-app @kind logic */
import type { IconName } from '@drizztdourden08/tessera';
import { FALLBACK_ICON, KIND_ICON } from './model.constants';

const kindIcon = (kind: string): IconName => KIND_ICON[kind] ?? FALLBACK_ICON;

export { kindIcon };
