/* @layer renderer-app @kind constants */
import type { ShortcutListItem } from '@drizztdourden08/tessera';
import type { RunStatus } from '../../../../data/kanban';
import type { RunOutcome, SummaryKey } from './run.type';

const RUN_OUTCOMES: readonly RunOutcome[] = [
  { status: 'skip', label: 'Skip', key: 's', variant: 'quiet' },
  { status: 'blocked', label: 'Blocked', key: 'b', variant: 'info' },
  { status: 'partial', label: 'Partial', key: 'm', variant: 'warning' },
  { status: 'fail', label: 'Fail', key: 'f', variant: 'danger' },
  { status: 'pass', label: 'Pass', key: 'p', variant: 'success' },
];

const NEEDS_NOTE: readonly RunStatus[] = ['fail', 'partial', 'blocked'];

const NOTE_MAX = 300;

const SUMMARY_ROWS: readonly (readonly [SummaryKey, string])[] = [
  ['pass', 'Passed'],
  ['fail', 'Failed'],
  ['partial', 'Partial'],
  ['blocked', 'Blocked'],
  ['skip', 'Skipped'],
  ['notRun', 'Not run'],
];

const FLAGGED_STATUSES: readonly RunStatus[] = ['fail', 'partial', 'blocked'];

const RUN_KEY_ITEMS: readonly ShortcutListItem[] = [
  { description: 'Previous case', keys: 'left' },
  { description: 'Next case', keys: 'right' },
  ...RUN_OUTCOMES.map((outcome) => ({ description: outcome.label, keys: outcome.key })),
];

const EMPTY_SUMMARY = { pass: 0, fail: 0, partial: 0, blocked: 0, skip: 0, notRun: 0, total: 0 };

const ENDED_STAGES: readonly string[] = ['reported', 'passed'];

const STRIP_SELECTOR = '[role="toolbar"]';

export { EMPTY_SUMMARY, ENDED_STAGES, FLAGGED_STATUSES, NEEDS_NOTE, NOTE_MAX, RUN_KEY_ITEMS, RUN_OUTCOMES, STRIP_SELECTOR, SUMMARY_ROWS };
