/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { boardMenu } from '../../src/screens/overview/boards/model/board-menu';
import { record } from './helpers';

const master = { name: 'm', title: null, cmd: null, current: true, window: false, play: 'm', updatedAt: null, by: null };
const ids = (found: ReturnType<typeof boardMenu>) => found.map((entry) => entry.id);

describe('P3 the menu of a board card', { tags: ['critical'] }, () => {
  it('offers open, send, play, edit and archive for a live board with a master', () => {
    expect(ids(boardMenu(record('a', { master })))).toEqual(['open', 'send', 'play', 'edit', 'archive']);
  });

  it('leaves out play and stop with no master', () => {
    expect(ids(boardMenu(record('a')))).toEqual(['open', 'send', 'edit', 'archive']);
  });

  it('offers stop instead of play while the master runs', () => {
    const running = { ...record('a', { master }), masterRunning: true };
    expect(ids(boardMenu(running))).toEqual(['open', 'send', 'stop', 'edit', 'archive']);
  });

  it('offers unarchive, not archive, send or play, for an archived board', () => {
    const archived = record('a', { master }, { archivedAt: '2026-11-01T00:00:00.000Z' });
    expect(ids(boardMenu(archived))).toEqual(['open', 'edit', 'unarchive']);
  });

  it('marks archive as a danger action', () => {
    expect(boardMenu(record('a')).find((entry) => entry.id === 'archive')?.tone).toBe('danger');
  });
});
