/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';
import type { ButtonVariant, CardTone, StackedBarColor, TagLook } from '@drizztdourden08/tessera';
import type { TestOutcome } from '../../../../data/posts';
import type { BundleState } from '../../../../data/types';
import type { Counts, WakeChip } from './model';

const TEST_STATUSES = defineStatuses({
  pending: { label: 'To run', tone: 'warning' },
  pass: { label: 'Pass', tone: 'success' },
  fail: { label: 'Fail', tone: 'danger' },
  partial: { label: 'Partial', tone: 'warning' },
  blocked: { label: 'Blocked', tone: 'info' },
  moved: { label: 'Moved', tone: 'neutral' },
});

const TEST_TAGS: Record<string, TagLook> = {
  pending: { variant: 'urgency', color: 'warning' },
  pass: { variant: 'urgency', color: 'success' },
  fail: { variant: 'urgency', color: 'danger' },
  partial: { variant: 'urgency', color: 'warning' },
  blocked: { variant: 'urgency', color: 'info' },
  moved: { variant: 'normal', color: 'neutral' },
};

const RESULT_TONES: Record<string, CardTone> = {
  pass: 'success',
  fail: 'danger',
  partial: 'warning',
  blocked: 'info',
};

const WAKE_TAGS: Record<WakeChip['tone'], TagLook> = {
  success: { variant: 'urgency', color: 'info' },
  warning: { variant: 'urgency', color: 'warning' },
  neutral: { variant: 'normal', color: 'neutral' },
};

const BUNDLE_TAGS: Record<BundleState, TagLook> = {
  draft: { variant: 'urgency', color: 'warning' },
  ready: { variant: 'urgency', color: 'success' },
  closed: { variant: 'normal', color: 'neutral' },
};

const BUNDLE_TIPS: Record<BundleState, string> = {
  draft: 'Draft: not built yet. Its tests can be written, but there is no Play and it cannot be handed to you.',
  ready: 'Ready: built and testable.',
  closed: 'Closed: finished and landed.',
};

const OUTCOMES: { id: TestOutcome; label: string; variant: ButtonVariant }[] = [
  { id: 'pass', label: 'pass', variant: 'success' },
  { id: 'fail', label: 'fail', variant: 'danger' },
  { id: 'partial', label: 'partial', variant: 'warning' },
  { id: 'blocked', label: 'blocked', variant: 'info' },
];

const PROGRESS_SEGMENTS: { id: keyof Counts; label: string; color: StackedBarColor }[] = [
  { id: 'pass', label: 'pass', color: 'success' },
  { id: 'fail', label: 'fail', color: 'danger' },
  { id: 'partial', label: 'partial', color: 'warning' },
  { id: 'blocked', label: 'blocked', color: 'info' },
  { id: 'run', label: 'to run', color: 'neutral' },
];

const COMPOSE_HINTS: Record<string, string> = {
  fail: 'What failed? The thread reads this.',
  partial: 'What works and what does not?',
  blocked: 'What blocks you?',
};

const ANCHOR_PATTERN = /^T-\d+$/;

const TYPING_SELECTOR = 'input, textarea, select, [contenteditable]';

const WAKE_TICK_MS = 5000;

const OUTCOME_BUTTON_SELECTOR = '[role="group"][aria-label^="Run "] button:not(:disabled)';

const RUN_ROW_SELECTOR = '[data-test][data-to-run]';

const KEY_HINT = 'keys 1 pass, 2 fail, 3 partial, 4 blocked (2 to 4 ask what you saw first)';

const NOTE_FOLD_CHARS = 160;

const NO_BUILD_VALUE = '(no build)';

const FIND_TEXT = 'Search id or title';

const ANY_CHOICE = 'any';


const FILTER_SWITCHES: ['needs' | 'conflict' | 'archived', string][] = [
  ['needs', 'needs you'],
  ['conflict', 'conflict'],
  ['archived', 'archived'],
];

const PASSED_TIP = 'The passed tests of open bundles.';

const NO_TESTS_TEXT = 'No test yet. Threads document every test here with coord test.';

const NO_MATCH_TEXT = 'No test matches the filters.';

const CASE_KIND = 'test-case';

const NO_PLANS_TEXT = 'No test plan yet.';

const NO_PLAN_GROUP = 'Not in a plan';

const CLOSED_PLANS = 'Closed plans';

const NO_CASES_TIP = 'Add cases to this plan, then start a run.';

const NOTE_DIALOG_TEXT = 'The note is posted with the outcome you pick. Fail, partial and blocked need one; pass does not.';

const MODE_OPTIONS = [
  { value: 'auto', label: 'auto' },
  { value: 'manual', label: 'manual' },
];

export {
  ANCHOR_PATTERN, ANY_CHOICE, BUNDLE_TAGS, BUNDLE_TIPS, CASE_KIND, CLOSED_PLANS, COMPOSE_HINTS, FILTER_SWITCHES, FIND_TEXT, KEY_HINT, NO_BUILD_VALUE, MODE_OPTIONS, NO_CASES_TIP, NO_MATCH_TEXT, NO_PLAN_GROUP, NO_PLANS_TEXT, NO_TESTS_TEXT, NOTE_DIALOG_TEXT, NOTE_FOLD_CHARS, OUTCOME_BUTTON_SELECTOR, OUTCOMES, PASSED_TIP, PROGRESS_SEGMENTS,
  RESULT_TONES, RUN_ROW_SELECTOR, TEST_STATUSES, TEST_TAGS, TYPING_SELECTOR, WAKE_TAGS, WAKE_TICK_MS,
};
