/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { Doc } from './kanban-harness';
import { MIN, ago, cleanKanban, item, mountKanban } from './kanban-harness';
import { caseDoc } from './kanban-case-doc';

cleanKanban();

const runCase = (id: string): Doc =>
  caseDoc(id, { item: 'I-9', covers: null, context: null, changed: [], setup: [{ label: 'Start', text: 'launch it' }], notes: [], ac: [] });

const CASES = ['T-1', 'T-2', 'T-3'];

const run = (results: Doc[] = [], extra: Doc = {}): Doc =>
  item('I-11', { type: 'test-run', stage: 'running', status: 'doing', createdBy: 'human', startedAt: ago(MIN), fields: { plan: 'I-1', build: 'b1', results }, ...extra });

const outcome = (test: string, status: string, note = ''): Doc => ({ test, status, note, at: ago(MIN) });

const plan = (): Doc => item('I-1', { type: 'test-plan', stage: 'running', fields: { cases: CASES } });

const board = (runs: Doc[] = []) => ({ items: [plan(), ...runs], tests: CASES.map(runCase) });

const mountRun = async (runs: Doc[] = []) => {
  const mounted = await mountKanban(board(runs), undefined, '#I-1/runs');
  const dialog = await screen.findByRole('dialog', { name: 'Work item' });
  return { ...mounted, dialog };
};

const button = (dialog: HTMLElement, name: string): HTMLElement => within(dialog).getByRole('button', { name });

const onCase = async (dialog: HTMLElement, id: string): Promise<void> => {
  await waitFor(() => expect(within(dialog).getByRole('group', { name: `Case ${id}` })).toBeTruthy());
};

describe('I-70 P7 starting a run', { tags: ['standard'] }, () => {
  it('offers Start run on a plan without a run and posts plan-run-start on the plan', async () => {
    const { dialog, posted } = await mountRun();
    fireEvent.click(await within(dialog).findByRole('button', { name: 'Start run' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'plan-run-start', author: 'human', target: { c: 'items', id: 'I-1' } });
  });
});

describe('I-70 P7 stepping through the cases', { tags: ['standard'] }, () => {
  it('shows the current case with its setup, steps and expected lines, and the position', async () => {
    const { dialog } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    expect(within(dialog).getByText('case 1 of 3')).toBeTruthy();
    expect(within(dialog).getByText('Case T-1')).toBeTruthy();
    expect(within(dialog).getByText('Start: launch it')).toBeTruthy();
    expect(within(dialog).getByText('open it')).toBeTruthy();
    expect(within(dialog).getByText('it opens')).toBeTruthy();
    expect(within(dialog).getByText('it breaks')).toBeTruthy();
  });

  it('moves with Next and Previous without posting, and disables Previous on the first case', async () => {
    const { dialog, posted } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    expect(button(dialog, 'Previous').hasAttribute('disabled')).toBe(true);
    fireEvent.click(button(dialog, 'Next'));
    await onCase(dialog, 'T-2');
    fireEvent.click(button(dialog, 'Previous'));
    await onCase(dialog, 'T-1');
    expect(posted).toHaveLength(0);
  });

  it('jumps to a case from the outcome strip', async () => {
    const { dialog } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    fireEvent.click(within(dialog).getByRole('button', { name: /^T-3/ }));
    await onCase(dialog, 'T-3');
    expect(within(dialog).getByText('case 3 of 3')).toBeTruthy();
  });

  it('walks the arrow keys', async () => {
    const { dialog } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    fireEvent.keyDown(document.body, { key: 'ArrowRight' });
    await onCase(dialog, 'T-2');
    fireEvent.keyDown(document.body, { key: 'ArrowLeft' });
    await onCase(dialog, 'T-1');
  });
});

describe('I-70 P7 recording', { tags: ['standard'] }, () => {
  it('posts plan-run-record for a pass and moves to the next case', async () => {
    const { dialog, posted } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    fireEvent.click(button(dialog, 'Pass'));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'plan-run-record', target: { c: 'items', id: 'I-1' }, test: 'T-1', result: 'pass', note: '' });
    await onCase(dialog, 'T-2');
  });

  it('records skip with its key', async () => {
    const { dialog, posted } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    fireEvent.keyDown(document.body, { key: 's' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'plan-run-record', test: 'T-1', result: 'skip' });
  });

  it('refuses a fail without a note, then sends it with the note', async () => {
    const { dialog, posted } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    fireEvent.click(button(dialog, 'Fail'));
    expect((await within(dialog).findByRole('alert')).textContent).toBe('A note is required for fail.');
    expect(posted).toHaveLength(0);
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Note' }), { target: { value: 'no Approve button' } });
    expect(within(dialog).queryByRole('alert')).toBeNull();
    fireEvent.click(button(dialog, 'Fail'));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'plan-run-record', test: 'T-1', result: 'fail', note: 'no Approve button' });
    await onCase(dialog, 'T-2');
  });

  it.each(['Partial', 'Blocked'])('also asks for a note on %s', async (name) => {
    const { dialog, posted } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    fireEvent.click(button(dialog, name));
    expect((await within(dialog).findByRole('alert')).textContent).toBe(`A note is required for ${name.toLowerCase()}.`);
    expect(posted).toHaveLength(0);
  });

  it('does not take letters typed in the note field for outcomes', async () => {
    const { dialog, posted } = await mountRun([run()]);
    await onCase(dialog, 'T-1');
    fireEvent.keyDown(within(dialog).getByRole('textbox', { name: 'Note' }), { key: 'p' });
    expect(posted).toHaveLength(0);
  });
});

describe('I-70 P7 reload and end', { tags: ['standard'] }, () => {
  it('resumes at the first case without a result from the test run', async () => {
    const { dialog } = await mountRun([run([outcome('T-1', 'pass'), outcome('T-2', 'skip')])]);
    await onCase(dialog, 'T-3');
    expect(within(dialog).getByText('case 3 of 3')).toBeTruthy();
    expect(within(dialog).getByText('1 not run yet; they count as not run.')).toBeTruthy();
  });

  it('posts plan-run-end with End run', async () => {
    const { dialog, posted } = await mountRun([run([outcome('T-1', 'pass')])]);
    await onCase(dialog, 'T-2');
    fireEvent.click(button(dialog, 'End run'));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'plan-run-end', target: { c: 'items', id: 'I-1' } });
  });

  it('shows the summary of an ended run with counts, the flagged cases and a way to start again', async () => {
    const ended = run([outcome('T-1', 'pass'), outcome('T-2', 'fail', 'no button')], { stage: 'reported', endedAt: ago(MIN) });
    const { dialog, posted } = await mountRun([ended]);
    const group = await within(dialog).findByRole('group', { name: 'Summary of run 11' });
    expect(within(group).getByText('Run 11 ended: 3 cases')).toBeTruthy();
    for (const [label, value] of [['Passed', '1'], ['Failed', '1'], ['Partial', '0'], ['Blocked', '0'], ['Skipped', '0'], ['Not run', '1']] as const) {
      expect(within(group).getByText(label).parentElement?.textContent).toContain(value);
    }
    expect(within(group).getByText('no button')).toBeTruthy();
    fireEvent.click(within(group).getByRole('button', { name: 'Start a new run' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'plan-run-start' });
  });

  it('moves from the runner to the summary when the board ends the run', async () => {
    const { dialog, change } = await mountRun([run([outcome('T-1', 'pass')])]);
    await onCase(dialog, 'T-2');
    change('items', 'I-11', run([outcome('T-1', 'pass')], { stage: 'reported', endedAt: ago(0) }));
    expect(await within(dialog).findByRole('group', { name: 'Summary of run 11' })).toBeTruthy();
  });
});
