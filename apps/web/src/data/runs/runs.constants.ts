/* @layer renderer-app @kind constants */
import type { RunStatus, RunTone } from './runs.type';

const LIVE_RUN_STATES: readonly RunStatus[] = ['queued', 'running'];

const QUEUED_LABEL = 'Queued, waiting for a free slot';

const LAUNCHING_LABEL = 'Launching...';

const RUNNING_LABEL = 'running';

const END_TONES: Record<RunStatus, RunTone> = { queued: 'warning', running: 'success', exited: 'info', stopped: 'neutral', failed: 'danger' };

const ANSI_PATTERN = new RegExp(`${String.fromCharCode(27)}\\[[0-?]*[ -/]*[@-~]`, 'g');

const LINE_MAX_CHARS = 120;

const RUN_LOG_TAIL = 300;

const RUN_LOG_FULL = 4000;

const RUN_LOG_FLUSH_MS = 40;

export { ANSI_PATTERN, END_TONES, LAUNCHING_LABEL, LINE_MAX_CHARS, LIVE_RUN_STATES, QUEUED_LABEL, RUN_LOG_FLUSH_MS, RUN_LOG_FULL, RUN_LOG_TAIL, RUNNING_LABEL };
