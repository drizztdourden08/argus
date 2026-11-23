/* @layer renderer-app @kind logic */
import { NO_LIVE } from './board-signals.constants';
import { signalText } from './signal-text';
import type { Raw } from './board-signals.type';
import type { BoardLive, BoardRecord, BoardSignal } from './board.type';

const withText = (raw: Raw): BoardSignal => ({ ...raw, text: signalText(raw.id, raw.count) });

const boardSignals = (record: BoardRecord, live: BoardLive = NO_LIVE): BoardSignal[] => {
  if (record.archived) return [withText({ id: 'archived', tone: 'muted', count: 1, label: 'archived', tab: 'overview' })];
  const { counts } = record;
  const all: Raw[] = [
    { id: 'needs-you', tone: 'warn', count: counts.needsYou, label: 'needs you', tab: 'overview' },
    { id: 'failing', tone: 'bad', count: counts.testsFailing, label: 'failing tests', tab: 'tests' },
    { id: 'blocked', tone: 'bad', count: counts.blocked, label: 'blocked threads', tab: 'work' },
    { id: 'stalled', tone: 'bad', count: live.stalls, label: live.stallLines?.join('; ') ?? 'stalled', tab: 'activity' },
    { id: 'sync-due', tone: 'warn', count: live.syncDue ? 1 : 0, label: 'sync due', tab: 'overview' },
    { id: 'master-missing', tone: 'neutral', count: record.masterMissing ? 1 : 0, label: 'master missing', tab: 'builds' },
    { id: 'paused', tone: 'muted', count: counts.paused, label: 'paused threads', tab: 'work' },
  ];
  return all.filter((signal) => signal.count > 0).map(withText);
};

export { boardSignals };
