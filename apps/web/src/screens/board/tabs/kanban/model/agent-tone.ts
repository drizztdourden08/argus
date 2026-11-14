/* @layer renderer-app @kind logic */
import type { TagUrgencyColor } from '@drizztdourden08/tessera';
import { AGENT_TONES } from '../kanban.constants';

const agentTone = (status: string | undefined): TagUrgencyColor => AGENT_TONES[status ?? ''] ?? 'info';

export { agentTone };
