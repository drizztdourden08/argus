/* @layer renderer-app @kind logic */
import type { StatusTone } from '@drizztdourden08/tessera';
import { ACTIVITY_TONES } from './model.constants';

const activityTone = (activity: string | null | undefined): StatusTone => ACTIVITY_TONES[activity ?? ''] ?? 'neutral';

export { activityTone };
