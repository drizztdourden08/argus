/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Doc } from './kanban-harness';
import { DAY, MIN, ago, cleanKanban, item, mountKanban } from './kanban-harness';
import { caseDoc } from './kanban-case-doc';

cleanKanban();

const done = (id: string): Doc => {
  const result = { status: 'fail', text: 'it broke', by: 'human', at: ago(MIN) };
  return caseDoc(id, { status: 'fail', result, runs: [{ status: 'pass', text: 'first try held', by: 'lead', at: ago(DAY) }, result] });
};

const board = (tests: Doc[]) => ({ items: [item('I-1', { ac: [{ text: 'The sage speaks', done: true }] })], tests });

const mountCase = async (tests: Doc[], tab = '') => {
  const mounted = await mountKanban(board(tests), undefined, `#T-1${tab}`);
  const dialog = await screen.findByRole('dialog', { name: 'Test detail' });
  return { ...mounted, dialog };
};

describe('P5 the test case pane', { tags: ['standard'] }, () => {
  it('has the tabs of the test case kind, from the registry', async () => {
    const { dialog } = await mountCase([caseDoc('T-1')]);
    const names = within(dialog).getAllByRole('tab').map((tab) => tab.textContent);
    for (const name of ['Overview', 'Result', 'Links', 'Discussion', 'Media', 'Activity']) expect(names.some((text) => text.startsWith(name))).toBe(true);
  });

  it('shows the facts, setup, steps, expected lines and criteria on the overview', async () => {
    const { dialog } = await mountCase([caseDoc('T-1')]);
    expect(within(dialog).getByRole('region', { name: 'Setup' })).toBeTruthy();
    expect(within(within(dialog).getByRole('list', { name: 'Steps' })).getAllByRole('listitem')).toHaveLength(2);
    expect(within(dialog).getByRole('region', { name: 'Expected' })).toBeTruthy();
    expect(within(dialog).getByRole('group', { name: 'Covers on I-1' })).toBeTruthy();
    expect(within(dialog).getByText('The game read the wrong level.')).toBeTruthy();
    expect(within(dialog).queryByText(/is not shown yet/)).toBeNull();
  });

  it('shows the mode and item selects of the facts', async () => {
    const { dialog } = await mountCase([caseDoc('T-1')]);
    expect(within(dialog).getByRole('combobox', { name: 'Mode' })).toBeTruthy();
    expect(within(dialog).getByRole('combobox', { name: 'Item' })).toBeTruthy();
  });
});

describe('P5 the result tab of the test case pane', { tags: ['standard'] }, () => {
  it('holds the outcome buttons and records a pass from them', async () => {
    const { dialog, posted } = await mountCase([caseDoc('T-1')], '/result');
    fireEvent.click(within(await within(dialog).findByRole('group', { name: 'Run T-1' })).getByRole('button', { name: 'pass' }));
    const results = () => posted.filter((call) => call.url.endsWith('/result'));
    await waitFor(() => expect(results()).toHaveLength(1));
    expect(results()[0]?.body).toMatchObject({ outcome: 'pass' });
  });

  it('asks for a note on a fail', async () => {
    const { dialog } = await mountCase([caseDoc('T-1')], '/result');
    fireEvent.click(within(await within(dialog).findByRole('group', { name: 'Run T-1' })).getByRole('button', { name: 'fail' }));
    expect(within(dialog).getByRole('textbox', { name: 'Note for T-1' })).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Send fail' })).toBeTruthy();
  });

  it('shows the result, the history newest first, the notes and Run again for a done test', async () => {
    const { dialog } = await mountCase([done('T-1')], '/result');
    expect(await within(dialog).findByRole('group', { name: 'Result' })).toBeTruthy();
    const rows = within(within(dialog).getByRole('group', { name: 'Results history · 2' })).getAllByRole('listitem');
    expect(rows[0]?.textContent).toContain('it broke');
    expect(rows[1]?.textContent).toContain('first try held');
    expect(within(dialog).getByText('Only the lead can tick I-1.')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Run T-1 again' })).toBeTruthy();
  });
});

describe('P5 the media tab of the test case pane', { tags: ['standard'] }, () => {
  it('shows the proofs of the test and the upload form', async () => {
    const proofs = [{ n: 1, caption: 'the sage at the altar', url: '/files/alpha/proofs/T-1-1.png', at: ago(MIN) }];
    const { dialog } = await mountCase([caseDoc('T-1', { proofs })], '/media');
    const evidence = await within(dialog).findByRole('group', { name: 'Evidence · 1' });
    expect(within(evidence).getByRole('button', { name: 'Open the sage at the altar' })).toBeTruthy();
    expect(within(evidence).getByRole('textbox', { name: 'Caption for T-1' })).toBeTruthy();
  });
});
