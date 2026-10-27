/* @layer renderer-app @kind constants */
import type { IconName, TextProps } from '@drizztdourden08/tessera';

const NOTIFY_LEVELS = ['info', 'ping', 'urgent'] as const;

const LEVEL_LOOK = {
  info: { variant: 'normal', color: 'neutral' },
  ping: { variant: 'urgency', color: 'info' },
  urgent: { variant: 'urgency', color: 'danger' },
} as const;

const NOTE_ICON: Readonly<Record<(typeof NOTIFY_LEVELS)[number], IconName>> = { info: 'file-text', ping: 'bell', urgent: 'triangle-alert' };

const NOTE_TONE: Readonly<Record<(typeof NOTIFY_LEVELS)[number], TextProps['tone']>> = { info: 'muted', ping: 'info', urgent: 'danger' };

const SETTING_PARTS = ['toast', 'sound'] as const;

export { LEVEL_LOOK, NOTE_ICON, NOTE_TONE, NOTIFY_LEVELS, SETTING_PARTS };
