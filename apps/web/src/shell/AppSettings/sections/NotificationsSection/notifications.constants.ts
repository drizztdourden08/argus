/* @layer renderer-app @kind constants */
import type { AutoLevels } from '../../../../data/notify';
import type { NotifyLevel } from '../../../../data/types';

interface LevelLabel {
  level: NotifyLevel;
  title: string;
}

interface AutoLabel {
  key: keyof AutoLevels;
  title: string;
}

const LEVELS: readonly LevelLabel[] = [
  { level: 'info', title: 'Info' },
  { level: 'ping', title: 'Ping' },
  { level: 'urgent', title: 'Urgent' },
];

const AUTO_PINGS: readonly AutoLabel[] = [
  { key: 'handed', title: 'A build is handed to you' },
  { key: 'ask', title: 'A thread asks a question' },
  { key: 'assigned', title: 'Assigned to you' },
  { key: 'verify', title: 'Ready to verify' },
  { key: 'run', title: 'A test plan is ready to run' },
];

export { AUTO_PINGS, LEVELS };
export type { AutoLabel, LevelLabel };
