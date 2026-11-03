/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { Entry } from '../../src/data/types';
import { buildWork, sectionOf } from '../../src/screens/board/feed-model';

const sent = (over: Partial<Entry> & { id: string }): Entry => ({
  kind: 'followup',
  author: 'human',
  thread: 'lead',
  status: 'new',
  text: 'also check the pane',
  createdAt: '2026-10-26T08:10:00.000Z',
  ...over,
});

const work = (entries: Entry[]) => buildWork({ entries, actions: [], tests: [], builds: [], log: [], items: [] });

describe('I-85 what the owner sent and no lead applied yet', { tags: ['critical'] }, () => {
  it('shows an unapplied followup and message in the section for the leads', () => {
    const result = work([sent({ id: 'F-1' }), sent({ id: 'M-2', kind: 'message' })]);
    expect(result.leads.map((card) => card.data.id).sort()).toEqual(['F-1', 'M-2']);
    expect(result.leads.every((card) => card.type === 'sent' && sectionOf(card) === 'leads')).toBe(true);
    expect(result.waiting).toEqual([]);
  });

  it('drops a followup once a lead applied it', () => {
    const result = work([sent({ id: 'F-1', appliedAt: '2026-10-26T08:20:00.000Z' }), sent({ id: 'F-2', status: 'done' })]);
    expect(result.leads).toEqual([]);
  });

  it('leaves what a lead wrote and other kinds out', () => {
    const result = work([sent({ id: 'F-1', author: 'lead' }), sent({ id: 'X-1', kind: 'answer' })]);
    expect(result.leads).toEqual([]);
  });
});
