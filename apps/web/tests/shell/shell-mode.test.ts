/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { boardsTarget, parseShellMode } from '../../src/shell/AppShell';
import { boardUrl } from '../../src/shell/behavior/board-url';
import type { Card } from '../../src/data/types';

const card = (key: string): Card => ({ key, name: key, state: 'running', root: null, url: `/p/${key}/` });

describe('I-106 the shell mode', { tags: ['critical'] }, () => {
  it('reads the mode and the board key from the path', () => {
    expect(parseShellMode('/')).toEqual({ mode: 'overview', boardKey: null });
    expect(parseShellMode('/p/rotp')).toEqual({ mode: 'boards', boardKey: 'rotp' });
    expect(parseShellMode('/p/rotp/')).toEqual({ mode: 'boards', boardKey: 'rotp' });
    expect(parseShellMode('/p/a%20b/')).toEqual({ mode: 'boards', boardKey: 'a b' });
    expect(parseShellMode('/p/')).toEqual({ mode: 'overview', boardKey: null });
  });

  it('sends Boards to the last board when it exists, else to the first card', () => {
    const cards = [card('argus'), card('rotp')];
    expect(boardsTarget('rotp', cards)).toBe('rotp');
    expect(boardsTarget('gone', cards)).toBe('argus');
    expect(boardsTarget(null, cards)).toBe('argus');
    expect(boardsTarget('rotp', [])).toBe('rotp');
    expect(boardsTarget(null, [])).toBeNull();
  });

  it('builds the board address with the hash kept', () => {
    expect(boardUrl('rotp')).toBe('/p/rotp/');
    expect(boardUrl('rotp', '#tests')).toBe('/p/rotp/#tests');
    expect(boardUrl('a b')).toBe('/p/a%20b/');
  });
});
