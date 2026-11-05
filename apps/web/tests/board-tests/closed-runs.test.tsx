/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cardOf, cleanAfterEach, isOpen, toggleOf } from './board-dom';
import { doneDoc, planDoc, runOn, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const results = [{ test: 'T-3', status: 'pass', note: '', at: 'x' }, { test: 'T-5', status: 'fail', note: 'no', at: 'x' }];

const closedBoard = () => ({
  group: 'run',
  items: [
    planDoc('I-9', ['T-1', 'T-2']),
    planDoc('I-8', ['T-3', 'T-4', 'T-5']),
    runOn('I-50', null),
    { ...runOn('I-51', null, { stage: 'passed', fields: { plan: 'I-8', build: null, results, summary: '' } }) },
  ],
  tests: [testDoc('T-1'), doneDoc('T-2', 'pass', 'ok'), testDoc('T-3'), testDoc('T-4'), doneDoc('T-5', 'fail', 'no')],
});

describe('S1 closed runs', { tags: ['standard'] }, () => {
  it('are out of the Tests tab by default', async () => {
    const { container } = await mountTab(closedBoard());
    expect(await screen.findByText('Test number 1')).toBeTruthy();
    expect(cardOf(container, 'I-51')).toBeNull();
    expect(screen.queryByText('Test number 4')).toBeNull();
  });

  it('show as a muted card with Show closed', async () => {
    const { container } = await mountTab(closedBoard());
    fireEvent.click(await screen.findByRole('button', { name: 'Show closed (1)' }));
    const card = cardOf(container, 'I-51');
    expect(card.textContent).toContain('passed');
    expect(within(card).getByRole('group', { name: '1 to run · 1 pass · 1 fail' })).toBeTruthy();
  });

  it('unfold to read-only rows: no outcome buttons, no Play, no Hand', async () => {
    const { container } = await mountTab(closedBoard());
    fireEvent.click(await screen.findByRole('button', { name: 'Show closed (1)' }));
    const card = cardOf(container, 'I-51');
    expect(isOpen(card)).toBe(false);
    fireEvent.click(toggleOf(card));
    expect(isOpen(card)).toBe(true);
    expect(within(card).getByText('Test number 4')).toBeTruthy();
    expect(within(card).queryByRole('button', { name: /^(pass|fail|partial|blocked)$|Play|Hand|again/ })).toBeNull();
  });

  it('hide again when the toggle is turned off', async () => {
    const { container } = await mountTab(closedBoard());
    fireEvent.click(await screen.findByRole('button', { name: 'Show closed (1)' }));
    expect(cardOf(container, 'I-51')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Hide closed (1)' }));
    expect(cardOf(container, 'I-51')).toBeNull();
  });
});
