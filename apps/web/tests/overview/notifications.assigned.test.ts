/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { ownerRows } from '../../src/screens/overview/notifications/model';

const card = (key: string, name: string, ownerItems?: { id: string; title: string; type: string; stage: string }[]) => ({ key, name, state: 'live', root: null, url: `/p/${key}/`, ownerItems });

describe('P8 the owner list model', { tags: ['critical'] }, () => {
  it('flattens the owner items of every card, sorted by project then item number', () => {
    const rows = ownerRows([
      card('rotp', 'RotP', [{ id: 'I-10', title: 'Ten', type: 'story', stage: 'review' }, { id: 'I-9', title: 'Nine', type: 'bug', stage: 'doing' }]),
      card('argus', 'Argus', [{ id: 'H-2', title: 'Look at the tray', type: 'todo', stage: 'pending' }]),
      card('quiet', 'Quiet'),
    ]);
    expect(rows.map((row) => `${row.project}:${row.id}`)).toEqual(['argus:H-2', 'rotp:I-9', 'rotp:I-10']);
    expect(rows[1]).toMatchObject({ name: 'RotP', title: 'Nine', type: 'bug', stage: 'doing' });
  });

  it('is empty when no card names the owner', () => {
    expect(ownerRows([card('rotp', 'RotP', []), card('argus', 'Argus')])).toEqual([]);
  });
});
