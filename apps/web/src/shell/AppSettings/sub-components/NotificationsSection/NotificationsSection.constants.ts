/* @layer renderer-app @kind constants */
import type { AutoLabel, LevelLabel } from './NotificationsSection.type';

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

const IDLE_LINE = 'Sends one ping through the same path as a real one.';

export { AUTO_PINGS, IDLE_LINE, LEVELS };
