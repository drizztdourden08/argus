/* @layer renderer-app @kind constants */
import type { SignalTone } from './model';
import type { LeadWords, UrgencyColor } from './boards.type';

const SIGNAL_URGENCY: Readonly<Partial<Record<SignalTone, UrgencyColor>>> = {
  bad: 'danger',
  warn: 'warning',
};

const LEAD_WORDS: Readonly<Record<'running' | 'idle' | 'paused' | 'not started', LeadWords>> = {
  running: { text: 'Lead working', tone: 'success' },
  idle: { text: 'Lead idle', tone: 'neutral' },
  paused: { text: 'Lead paused', tone: 'neutral' },
  'not started': { text: 'Not started', tone: 'neutral' },
};

const NEEDS_BOARD: readonly string[] = ['open', 'send', 'play', 'stop'];

export { LEAD_WORDS, NEEDS_BOARD, SIGNAL_URGENCY };
