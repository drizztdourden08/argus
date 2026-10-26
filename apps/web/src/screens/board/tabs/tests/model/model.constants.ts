/* @layer renderer-app @kind constants */
import type { RunEntry, RunTone } from './model.type';

const NO_BUNDLE = '(no build)';

const NOTE_MIN_CHARS = 3;

const NOTE_MAX_CHARS = 50000;

const LINE_MAX_CHARS = 120;

const CAPTION_MAX_CHARS = 120;

const DONE_FOLD_AFTER = 5;

const NOTE_OUTCOMES = ['fail', 'partial', 'blocked'];

const IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];

const IMAGE_MAX_BYTES = 10 * 1024 * 1024;

const ANSI_PATTERN = new RegExp(`${String.fromCharCode(27)}\\[[0-?]*[ -/]*[@-~]`, 'g');

const LIVE_RUN_STATES = ['queued', 'running'];

const QUEUED_LABEL = 'Queued, waiting for a free slot';

const LAUNCHING_LABEL = 'Launching...';

const RUNNING_LABEL = 'running';

const END_TONES: Record<RunEntry['status'], RunTone> = { queued: 'warning', running: 'success', exited: 'info', stopped: 'neutral', failed: 'danger' };

export {
  ANSI_PATTERN, CAPTION_MAX_CHARS, DONE_FOLD_AFTER, END_TONES, IMAGE_MAX_BYTES, IMAGE_TYPES, LAUNCHING_LABEL, LINE_MAX_CHARS, LIVE_RUN_STATES,
  NO_BUNDLE, NOTE_MAX_CHARS, NOTE_MIN_CHARS, NOTE_OUTCOMES, QUEUED_LABEL, RUNNING_LABEL,
};
