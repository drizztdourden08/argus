/* @layer renderer-app @kind constants */
const NOTIFY_LEVELS = ['info', 'ping', 'urgent'] as const;

const LEVEL_LOOK = {
  info: { variant: 'normal', color: 'neutral' },
  ping: { variant: 'urgency', color: 'info' },
  urgent: { variant: 'urgency', color: 'danger' },
} as const;

export { LEVEL_LOOK, NOTIFY_LEVELS };
