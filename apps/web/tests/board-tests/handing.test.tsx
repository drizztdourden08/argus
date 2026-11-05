/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from './board-dom';
import { planDoc, runOn, testDoc } from './fixtures';
import type { Doc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const board = (runs: Doc[]) => ({ group: 'run', items: [planDoc('I-9', ['T-1']), ...runs], tests: [testDoc('T-1')] });

describe('S1 handing a run to the owner', { tags: ['standard'] }, () => {
  it('Hand to me assigns the run to the owner, and Unhand clears it', async () => {
    const { calls } = await mountTab(board([runOn('I-50', null, { stage: 'ready' }), runOn('I-51', null, { assignee: 'owner' })]));
    fireEvent.click(await screen.findByRole('button', { name: 'Hand run TR-50 to you' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/rotp/entries', method: 'POST', body: { kind: 'edit', target: { c: 'items', id: 'I-50' }, set: { assignee: 'owner' } } });
    fireEvent.click(screen.getByRole('button', { name: 'Stop handing run TR-51 to you' }));
    await waitFor(() => expect(calls).toHaveLength(2));
    expect(calls[1]?.body).toMatchObject({ kind: 'edit', target: { c: 'items', id: 'I-51' }, set: { assignee: null } });
  });

  it('is not offered for a draft that is not handed', async () => {
    await mountTab(board([runOn('I-52', null, { stage: 'draft' })]));
    await screen.findByText('Test number 1');
    expect(screen.queryByRole('button', { name: /Hand run/ })).toBeNull();
  });
});
