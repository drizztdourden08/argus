/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cardOf, cleanAfterEach, detailsOf, tagsOf } from './board-dom';
import { buildDoc, doneDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

describe('the order and the folds of bundle cards', () => {
  it('put the handed bundle first, name it, and open it', async () => {
    const { container } = await mountTab({
      handed: 'b2',
      builds: [buildDoc('b1'), buildDoc('b2')],
      tests: [testDoc('T-1', { build: 'b1' }), doneDoc('T-2', 'pass', 'ok', { build: 'b2' })],
    });
    await screen.findByText('Test T-1');
    expect(tagsOf(container)).toEqual(['b2', 'b1']);
    expect(within(cardOf(container, 'b2')).getByText('handed to you')).toBeTruthy();
    expect(detailsOf(cardOf(container, 'b2')).open).toBe(true);
  });

  it('open for tests to run and fold when finished, and a click on the header overrides that', async () => {
    const { container } = await mountTab({
      builds: [buildDoc('todo'), buildDoc('finished')],
      tests: [testDoc('T-1', { build: 'todo' }), doneDoc('T-2', 'pass', 'ok', { build: 'finished' })],
    });
    await screen.findByText('Test T-1');
    expect(detailsOf(cardOf(container, 'todo')).open).toBe(true);
    const finished = cardOf(container, 'finished');
    expect(detailsOf(finished).open).toBe(false);
    fireEvent.click(finished.querySelector('summary') as Element);
    expect(detailsOf(finished).open).toBe(true);
  });

  it('show a declared bundle with no test yet, and the tests without a bundle last', async () => {
    const { container } = await mountTab({ builds: [buildDoc('empty-1', { title: 'Nothing yet' })], tests: [testDoc('T-1', { build: null })] });
    await screen.findByText('Test T-1');
    expect(tagsOf(container)).toEqual(['empty-1', '(no build)']);
    const empty = cardOf(container, 'empty-1');
    expect(within(empty).getByText('Nothing yet')).toBeTruthy();
    expect(within(empty).getByText('No test in this bundle yet.')).toBeTruthy();
    expect(within(cardOf(container, '(no build)')).getByText('No bundle')).toBeTruthy();
  });

  it('say draft or ready with the tooltip meaning', async () => {
    const { container } = await mountTab({
      builds: [buildDoc('d1', { status: 'draft' }), buildDoc('r1')],
      tests: [testDoc('T-1', { build: 'd1' }), testDoc('T-2', { build: 'r1' })],
    });
    await screen.findByText('Test T-1');
    const draft = cardOf(container, 'd1').querySelector('[data-status="draft"]');
    expect(draft?.textContent).toBe('Draft');
    expect(draft?.getAttribute('title')).toMatch(/not built yet/);
    expect(cardOf(container, 'r1').querySelector('[data-status="ready"]')?.textContent).toBe('Ready');
  });
});

describe('the sections of a bundle card', () => {
  it('take a test in live: a change moves it from To run to Done', async () => {
    const { change } = await mountTab({ builds: [buildDoc('bundle-1')], tests: [testDoc('T-1')] });
    const toRun = await screen.findByRole('group', { name: 'To run' });
    expect(within(toRun).getByText('Test T-1')).toBeTruthy();
    change('tests', 'T-1', doneDoc('T-1', 'pass', 'ok'));
    const done = await screen.findByRole('group', { name: 'Done' });
    expect(within(done).getByText('Test T-1')).toBeTruthy();
    expect(within(screen.getByRole('group', { name: 'To run' })).getByText('Nothing left to run.')).toBeTruthy();
  });

  it('fold more than five done tests to the newest, with a toggle for all', async () => {
    const rows = Array.from({ length: 6 }, (_, index) => doneDoc(`T-${index + 1}`, 'pass', 'ok', {
      result: { status: 'pass', text: 'ok', by: 'human', at: `2026-10-26T0${index}:00:00.000Z` },
    }));
    await mountTab({ builds: [buildDoc('bundle-1')], tests: rows });
    const done = await screen.findByRole('group', { name: 'Done' });
    expect(within(done).getByText('Show all 6 done')).toBeTruthy();
    const outside = done.querySelector('ul') as HTMLElement;
    expect([...outside.querySelectorAll('[data-test]')].map((row) => row.getAttribute('data-test'))).toEqual(['T-6']);
  });

  it('list five done tests with no toggle', async () => {
    const rows = Array.from({ length: 5 }, (_, index) => doneDoc(`T-${index + 1}`, 'pass', 'ok'));
    await mountTab({ builds: [buildDoc('bundle-1')], tests: rows });
    const done = await screen.findByRole('group', { name: 'Done' });
    expect(within(done).queryByText(/Show all/)).toBeNull();
    expect(done.querySelectorAll('[data-test]')).toHaveLength(5);
  });

  it('say there is no test at all, and read the archived and deleted ones as absent', async () => {
    await mountTab({ tests: [testDoc('T-1', { archivedAt: 'then' }), testDoc('T-2', { deletedAt: 'then' })] });
    expect(await screen.findByText(/No test yet/)).toBeTruthy();
    expect(screen.queryByText('Test T-1')).toBeNull();
    expect(screen.queryByText('Test T-2')).toBeNull();
  });
});
