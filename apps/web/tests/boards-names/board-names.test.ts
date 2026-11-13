/* @layer renderer-app @kind test */
import { beforeEach, describe, expect, it } from 'vitest';
import { boardName, findBoard, migrateStoredKeys } from '../../src/data/boards';
import { boardsStore } from '../../src/data/boards/boards-store';

const BOARDS = [
  { id: 'b4k7qz2', name: 'ShardTide', aliases: ['geo-survivors'], role: null },
  { id: 'argus', name: 'Argus', aliases: [], role: 'argus' },
];

describe('board names', { tags: ['critical'] }, () => {
  beforeEach(() => boardsStore.set(BOARDS));

  it('finds a name by id', () => {
    expect(boardName('b4k7qz2')).toBe('ShardTide');
  });

  it('finds a name by alias', () => {
    expect(boardName('geo-survivors')).toBe('ShardTide');
    expect(findBoard(BOARDS, 'geo-survivors')?.id).toBe('b4k7qz2');
  });

  it('returns the input when the board is unknown', () => {
    expect(boardName('nobody')).toBe('nobody');
  });
});

describe('stored key migration', { tags: ['critical'] }, () => {
  beforeEach(() => localStorage.clear());

  it('rewrites an alias value to the id and leaves an id alone', () => {
    localStorage.setItem('last', 'geo-survivors');
    migrateStoredKeys(localStorage, BOARDS, ['last'], []);
    expect(localStorage.getItem('last')).toBe('b4k7qz2');
    migrateStoredKeys(localStorage, BOARDS, ['last'], []);
    expect(localStorage.getItem('last')).toBe('b4k7qz2');
  });

  it('moves a per-board key from the alias to the id without overwriting an id key', () => {
    localStorage.setItem('coord.focus.geo-survivors', 'a');
    migrateStoredKeys(localStorage, BOARDS, [], ['coord.focus']);
    expect(localStorage.getItem('coord.focus.b4k7qz2')).toBe('a');
    expect(localStorage.getItem('coord.focus.geo-survivors')).toBeNull();
    localStorage.setItem('coord.focus.geo-survivors', 'old');
    migrateStoredKeys(localStorage, BOARDS, [], ['coord.focus']);
    expect(localStorage.getItem('coord.focus.b4k7qz2')).toBe('a');
  });
});
