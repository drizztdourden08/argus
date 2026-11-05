/* @layer renderer-app @kind logic */
import type { BoardLive, BoardRecord, BoardSignal } from './board.type';

const NO_LIVE: BoardLive = { fault: null, stalls: 0, syncDue: false };

const faultLabel = (live: BoardLive): string => (live.fault?.reason === 'board-closed' ? 'board closed' : 'wake failed');

const boardSignals = (record: BoardRecord, live: BoardLive = NO_LIVE): BoardSignal[] => {
  if (record.archived) return [{ id: 'archived', tone: 'muted', count: 1, label: 'archived', tab: 'overview' }];
  const { counts } = record;
  const all: BoardSignal[] = [
    { id: 'needs-you', tone: 'warn', count: counts.needsYou, label: 'needs you', tab: 'overview' },
    { id: 'failing', tone: 'bad', count: counts.testsFailing, label: 'failing tests', tab: 'tests' },
    { id: 'blocked', tone: 'bad', count: counts.blocked, label: 'blocked threads', tab: 'work' },
    { id: 'stalled', tone: 'bad', count: live.stalls, label: 'stalled', tab: 'activity' },
    { id: 'fault', tone: 'bad', count: live.fault ? 1 : 0, label: faultLabel(live), tab: 'overview' },
    { id: 'sync-due', tone: 'warn', count: live.syncDue ? 1 : 0, label: 'sync due', tab: 'overview' },
    { id: 'master-missing', tone: 'neutral', count: record.masterMissing ? 1 : 0, label: 'master missing', tab: 'builds' },
    { id: 'paused', tone: 'muted', count: counts.paused, label: 'paused threads', tab: 'work' },
    { id: 'running', tone: 'good', count: record.masterRunning || record.lead === 'running' ? 1 : 0, label: 'running', tab: 'work' },
  ];
  return all.filter((signal) => signal.count > 0);
};

export { boardSignals };
