/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';
import type { ButtonVariant, StackedBarColor } from '@drizztdourden08/tessera';
import type { TestOutcome } from '../../../../data/posts';
import type { BundleState } from '../../../../data/types';
import type { Counts } from './model';

const TEST_STATUSES = defineStatuses({
  pending: { label: 'To run', tone: 'primary' },
  pass: { label: 'Pass', tone: 'success' },
  fail: { label: 'Fail', tone: 'danger' },
  partial: { label: 'Partial', tone: 'warning' },
  blocked: { label: 'Blocked', tone: 'info' },
  moved: { label: 'Moved', tone: 'neutral' },
});

const BUNDLE_STATUSES = defineStatuses({
  draft: { label: 'Draft', tone: 'warning' },
  ready: { label: 'Ready', tone: 'success' },
  closed: { label: 'Closed', tone: 'neutral' },
});

const BUNDLE_TIPS: Record<BundleState, string> = {
  draft: 'Draft: not built yet. Its tests can be written, but there is no Play and it cannot be handed to you.',
  ready: 'Ready: built and testable.',
  closed: 'Closed: finished and landed.',
};

const OUTCOMES: { id: TestOutcome; label: string; variant: ButtonVariant }[] = [
  { id: 'pass', label: 'Pass', variant: 'success' },
  { id: 'fail', label: 'Fail', variant: 'danger' },
  { id: 'partial', label: 'Partial', variant: 'warning' },
  { id: 'blocked', label: 'Blocked', variant: 'info' },
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

const RESULT_PLACEHOLDER = 'recorded on the board';

export { BUNDLE_STATUSES, BUNDLE_TIPS, COMPOSE_HINTS, OUTCOMES, PROGRESS_SEGMENTS, RESULT_PLACEHOLDER, TEST_STATUSES };
