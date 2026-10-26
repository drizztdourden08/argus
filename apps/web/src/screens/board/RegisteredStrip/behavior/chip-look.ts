/* @layer renderer-app @kind logic */
import type { RegistryRow } from '../../../../data/types';
import type { ChipLook } from '../RegisteredStrip.type';

const idleLook = (row: RegistryRow): ChipLook =>
  row.deaf
    ? { tone: 'warning', mark: 'warn', word: 'not listening', faulted: false }
    : { tone: 'neutral', mark: 'dot', word: 'not listening', faulted: false };

const baseLook = (row: RegistryRow): ChipLook => {
  if (row.state === 'fault' && row.fault) {
    return { tone: 'danger', mark: 'warn', word: row.fault.reason === 'board-closed' ? 'board closed' : 'wake failed', faulted: true };
  }
  if (row.state === 'waking') return { tone: 'warning', mark: 'spin', word: 'waking', faulted: false };
  if (row.state === 'busy') return { tone: 'info', mark: 'dot', word: 'working', faulted: false };
  if (row.state === 'listening') return { tone: 'success', mark: 'dot', word: '', faulted: false };
  return idleLook(row);
};

const chipLook = (row: RegistryRow): ChipLook => {
  const look = baseLook(row);
  const boardClosed = !look.faulted && row.boardRequired === true && row.boardOpen !== true;
  return boardClosed ? { ...look, word: [look.word, 'board closed'].filter(Boolean).join(' · ') } : look;
};

export { chipLook };
