/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { stallsOf, stallText } from '../../src/data/stalls';
import { STALLS } from './helpers/stalls';

const docs = Object.entries(STALLS).map(([id, data]) => ({ ...data, id }));

describe('K1 the stalls of a project', { tags: ['critical'] }, () => {
  it('keeps the stalls of one project, oldest first', () => {
    expect(stallsOf(docs, 'rotp').map((doc) => doc.kind)).toEqual(['unread', 'idle']);
  });

  it('keeps every project when none is asked for, oldest first', () => {
    expect(stallsOf(docs).map((doc) => doc.project)).toEqual(['argus', 'rotp', 'rotp']);
  });

  it('drops the stall head from the text of a chip', () => {
    expect(stallText('stall: rotp idle 12 min')).toBe('rotp idle 12 min');
    expect(stallText('no head')).toBe('no head');
  });
});
