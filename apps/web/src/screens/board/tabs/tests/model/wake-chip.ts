/* @layer renderer-app @kind logic */
import { HUMAN, WAKE_WAIT_MS } from './model.constants';
import type { WakeChip, WakeDoc, WakeInput } from './model.type';

const EARLIER_GRACE_MS = 2000;

const stamp = (iso: string | undefined | null): number => Date.parse(iso ?? '');

const speaker = (first: WakeDoc, wakes: readonly WakeDoc[]): WakeDoc => {
  if (first.status !== 'coalesced' || !first.into) return first;
  return wakes.find((wake) => wake.id === first.into) ?? first;
};

const staleWake = (first: WakeDoc, resultAt: string): boolean => {
  const wokeAt = stamp(first.at);
  const clickedAt = stamp(resultAt);
  return Number.isFinite(wokeAt) && Number.isFinite(clickedAt) && wokeAt < clickedAt - EARLIER_GRACE_MS;
};

const threadOf = (wake: WakeDoc, first: WakeDoc): string => wake.thread ?? first.thread ?? 'thread';

const heardChip = (wake: WakeDoc, first: WakeDoc): WakeChip | null => {
  if (wake.status !== 'delivered' || !(Number(wake.listeners) > 0)) return null;
  const thread = threadOf(wake, first);
  const merged = wake === first ? '' : ' (merged into a wake sent a moment earlier)';
  return { tone: 'success', pulse: false, label: `${thread} notified`, title: `${thread} is listening and was told about this result${merged}.` };
};

const waitingChip = (wake: WakeDoc, first: WakeDoc, now: number): WakeChip | null => {
  const age = now - (stamp(wake.at) || stamp(first.at) || 0);
  if (wake.status !== 'queued' || age >= WAKE_WAIT_MS) return null;
  return { tone: 'warning', pulse: true, label: `${threadOf(wake, first)} waking...`, title: 'The wake is sent when its short coalescing window closes.' };
};

const silentChip = (wake: WakeDoc, first: WakeDoc): WakeChip => {
  const thread = threadOf(wake, first);
  return { tone: 'neutral', pulse: false, label: `${thread} not listening`, title: `Nobody was listening for ${thread} when you clicked. It sees this result the next time it runs.` };
};

const wakeChip = (input: WakeInput): WakeChip | null => {
  const { test, wakes, latest, now } = input;
  const first = latest.get(test.id);
  const result = test.result;
  if (!first || result?.by !== HUMAN || staleWake(first, result.at)) return null;
  const wake = speaker(first, wakes);
  if (wake.status === 'coalesced') return null;
  return heardChip(wake, first) ?? waitingChip(wake, first, now) ?? silentChip(wake, first);
};

export { wakeChip };
