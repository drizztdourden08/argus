/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cardOf, cleanAfterEach, isOpen, tagsOf, toggleOf } from './board-dom';
import { doneDoc, planDoc, runOn, testDoc } from './fixtures';
import type { Doc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const board = (runs: Doc[], tests: Doc[], cases = ['T-1', 'T-2']) => ({ group: 'run', items: [planDoc('I-9', cases), ...runs], tests });

describe('S1 the order and the folds of run cards', { tags: ['standard'] }, () => {
  it('put the handed run first, name it, and open it', async () => {
    const { container } = await mountTab(board(
      [runOn('I-50', null, { stage: 'draft' }), runOn('I-51', null, { stage: 'ready', assignee: 'owner' })],
      [testDoc('T-1'), doneDoc('T-2', 'pass', 'ok')],
    ));
    await screen.findByText('Test number 1');
    expect(tagsOf(container)).toEqual(['I-51', 'I-50']);
    expect(within(cardOf(container, 'I-51')).getByText('handed to you')).toBeTruthy();
    expect(within(cardOf(container, 'I-51')).getByRole('button', { name: /Plan number 9/ })).toBeTruthy();
    expect(isOpen(cardOf(container, 'I-51'))).toBe(true);
  });

  it('open a running run, fold a draft, and a click on the header overrides that', async () => {
    const { container } = await mountTab(board([runOn('I-50', null), runOn('I-51', null, { stage: 'draft' })], [testDoc('T-1')]));
    await screen.findByText('Test number 1');
    expect(isOpen(cardOf(container, 'I-50'))).toBe(true);
    const draft = cardOf(container, 'I-51');
    expect(isOpen(draft)).toBe(false);
    fireEvent.click(toggleOf(draft));
    expect(isOpen(draft)).toBe(true);
  });

  it('show the counts of the run from its results, and say when no run exists', async () => {
    const results = [{ test: 'T-1', status: 'pass', note: '', at: 'x' }];
    const { container } = await mountTab(board([runOn('I-50', null, { fields: { plan: 'I-9', build: null, results, summary: '' } })], [testDoc('T-1'), testDoc('T-2')]));
    await screen.findByText('Test number 2');
    expect(within(cardOf(container, 'I-50')).getByRole('group', { name: '1 to run · 1 pass' })).toBeTruthy();
  });

  it('say there is no run yet', async () => {
    await mountTab(board([], [testDoc('T-1')]));
    expect(await screen.findByText(/No test run yet/)).toBeTruthy();
  });
});

describe('S1 the sections of a run card', { tags: ['standard'] }, () => {
  it('take a test in live: a change moves it from To run to Done', async () => {
    const { change } = await mountTab(board([runOn('I-50', null, { assignee: 'owner' })], [testDoc('T-1')], ['T-1']));
    const toRun = await screen.findByRole('group', { name: 'To run' });
    expect(within(toRun).getByText('Test number 1')).toBeTruthy();
    change('tests', 'T-1', doneDoc('T-1', 'pass', 'ok'));
    const done = await screen.findByRole('group', { name: 'Done' });
    expect(within(done).getByText('Test number 1')).toBeTruthy();
    expect(within(screen.getByRole('group', { name: 'To run' })).getByText('Nothing left to run.')).toBeTruthy();
  });

  it('fold more than five done tests to the newest, with a toggle for all', async () => {
    const rows = Array.from({ length: 6 }, (_, index) => doneDoc(`T-${index + 1}`, 'pass', 'ok', {
      result: { status: 'pass', text: 'ok', by: 'human', at: `2026-10-26T0${index}:00:00.000Z` },
    }));
    await mountTab(board([runOn('I-50', null, { assignee: 'owner' })], rows, rows.map((row) => String(Reflect.get(row, 'id')))));
    const done = await screen.findByRole('group', { name: 'Done' });
    expect(within(done).getByRole('button', { name: /6 done/ })).toBeTruthy();
    expect([...done.querySelectorAll('[data-test]')].map((row) => row.getAttribute('data-test'))).toEqual(['T-6']);
  });

  it('list five done tests with no toggle', async () => {
    const rows = Array.from({ length: 5 }, (_, index) => doneDoc(`T-${index + 1}`, 'pass', 'ok'));
    await mountTab(board([runOn('I-50', null, { assignee: 'owner' })], rows, rows.map((row) => String(Reflect.get(row, 'id')))));
    const done = await screen.findByRole('group', { name: 'Done' });
    expect(within(done).queryByRole('button', { name: /done/ })).toBeNull();
    expect(done.querySelectorAll('[data-test]')).toHaveLength(5);
  });

  it('say there is no test at all, and read the archived and deleted ones as absent', async () => {
    await mountTab({ tests: [testDoc('T-1', { archivedAt: 'then' }), testDoc('T-2', { deletedAt: 'then' })] });
    expect(await screen.findByText(/No test yet/)).toBeTruthy();
    expect(screen.queryByText('Test number 1')).toBeNull();
    expect(screen.queryByText('Test number 2')).toBeNull();
  });
});
