/* @layer renderer-app @kind constants */
import type { BlockedKind } from '../../../../../data/types';
import type { AttentionKind, AttentionSpec, Signals } from './attention.type';

const ATTENTION: readonly AttentionSpec[] = [
  { kind: 'question', icon: 'circle-help', color: 'danger', label: 'question', text: 'An open question waits for your answer.' },
  { kind: 'todo', icon: 'list-checks', color: 'danger', label: 'to-do', text: 'A to-do waits for you to do it.' },
  { kind: 'test-failing', icon: 'circle-x', color: 'danger', label: 'test failing', text: 'A test of the item failed, or is partial or blocked.' },
  { kind: 'dependency', icon: 'link', color: 'warning', label: 'dependency', text: 'The item waits on another item, or a request for it is blocked.' },
  { kind: 'blocked', icon: 'lock', color: 'warning', label: 'blocked', text: 'The item is blocked by something outside the board.' },
  { kind: 'test-pending', icon: 'flask', color: 'info', label: 'test pending', text: 'A test of the item has no result yet.' },
  { kind: 'decision', icon: 'split', color: 'info', label: 'decision', text: 'A decision waits to be picked up.' },
  { kind: 'quiet', icon: 'clock', color: 'warning', label: 'quiet', text: 'The item has had no update for longer than its status allows.' },
  { kind: 'agent', icon: 'robot', color: 'success', label: 'agent running', text: 'A sub-agent is working on the item.' },
];

const ATTENTION_OF: Readonly<Record<AttentionKind, AttentionSpec>> = Object.fromEntries(
  ATTENTION.map((spec) => [spec.kind, spec]),
) as Record<AttentionKind, AttentionSpec>;

const NO_SIGNALS: Signals = { failing: 0, pending: 0, agents: 0 };

const FAILING_STATUSES: readonly string[] = ['fail', 'partial', 'blocked'];

const WORKING_STATUS = 'working';

const DEPENDENCY_KINDS: readonly BlockedKind[] = ['dep-waits', 'dep-blocked'];

const LOCK_KINDS: readonly BlockedKind[] = ['item', 'test', 'todo', 'legacy'];

export { ATTENTION, ATTENTION_OF, DEPENDENCY_KINDS, FAILING_STATUSES, LOCK_KINDS, NO_SIGNALS, WORKING_STATUS };
