/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { allOpen, setRowOpen, toggleAll } from '../../src/screens/board/tabs/agents/model';

const IDS = ['A-1', 'A-2', 'A-3'];

describe('I-150 Agents tab: expand all toggle', { tags: ['critical'] }, () => {
  it('opens every row when none or only some are open', () => {
    expect([...toggleAll(new Set(), IDS)]).toEqual(IDS);
    expect([...toggleAll(new Set(['A-2']), IDS)]).toEqual(IDS);
  });

  it('closes every row when all are open', () => {
    expect(toggleAll(new Set(IDS), IDS).size).toBe(0);
  });

  it('reports all open only when every listed row is open, never for an empty list', () => {
    expect(allOpen(new Set(IDS), IDS)).toBe(true);
    expect(allOpen(new Set(['A-1']), IDS)).toBe(false);
    expect(allOpen(new Set(), [])).toBe(false);
  });

  it('opens and closes one row without touching the others and without mutating the set', () => {
    const start = new Set(['A-1']);
    const next = setRowOpen(start, 'A-2', true);
    expect([...next]).toEqual(['A-1', 'A-2']);
    expect([...start]).toEqual(['A-1']);
    expect([...setRowOpen(next, 'A-1', false)]).toEqual(['A-2']);
  });
});
