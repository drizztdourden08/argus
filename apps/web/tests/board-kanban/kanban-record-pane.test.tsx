/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { kindOf } from '../../src/data/registry';
import { PANE_TABS } from '../../src/shell/records/pane/pane.constants';
import { tabNames } from './kanban-pane-dom';
import { MIN, ago, cleanKanban, item, mountKanban, paneOf } from './kanban-harness';

cleanKanban();

const question = { id: 'Q-1', kind: 'question', item: 'I-1', author: 'worker', status: 'open', text: 'Ship it today?', createdAt: ago(10 * MIN) };
const testCase = { id: 'T-1', item: 'I-1', title: 'Wrong password', mode: 'manual', status: 'pass', ac: [1] };
const agentDoc = { id: 'A-1', name: 'coder', thread: 'worker', task: 'do it', kind: 'build', item: 'I-1', status: 'working', startedAt: ago(MIN), updatedAt: ago(MIN), rev: 1 };

const labelsOf = (id: string): string[] => (kindOf(id)?.tabs ?? []).filter((tab) => Object.hasOwn(PANE_TABS, tab)).map((tab) => PANE_TABS[tab]?.label ?? tab);

describe('I-85 P3 every registry tab has a body', { tags: ['critical'] }, () => {
  it.each(['task', 'bug', 'test-case', 'agent', 'question', 'build', 'worktree', 'run', 'release'])('%s lists only tabs the pane can draw', (id) => {
    const tabs = kindOf(id)?.tabs ?? [];
    expect(tabs.length).toBeGreaterThan(0);
    for (const tab of tabs) expect(PANE_TABS[tab], `${id} names the tab ${tab}`).toBeDefined();
  });
});

describe('I-85 P3 one pane host opens every kind from the address', { tags: ['critical'] }, () => {
  it('opens a work item with the tab the address names, never changing the page', async () => {
    await mountKanban({ items: [item('I-1', { title: 'Login' })] }, undefined, '?open=I-1&pane=links');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByRole('tab', { name: /^Links/ }).getAttribute('aria-selected')).toBe('true');
    expect(paneOf()).toEqual({ open: 'I-1', pane: 'links' });
  });

  it('opens an entry, with the tabs of its kind', async () => {
    await mountKanban({ items: [item('I-1')], entries: [question] }, undefined, '?open=Q-1');
    const dialog = await screen.findByRole('dialog', { name: 'Entry detail' });
    expect(tabNames(dialog).map((name) => name.replace(/\d+$/, ''))).toEqual(labelsOf('question'));
  });

  it('opens a test case, with the tabs of its kind', async () => {
    await mountKanban({ items: [item('I-1')], tests: [testCase] }, undefined, '?open=T-1');
    const dialog = await screen.findByRole('dialog', { name: 'Test detail' });
    expect(tabNames(dialog).map((name) => name.replace(/\d+$/, ''))).toEqual(labelsOf('test-case'));
  });

  it('opens an agent, with the tabs of its kind', async () => {
    await mountKanban({ items: [item('I-1')], agents: [agentDoc] }, undefined, '?open=A-1');
    const dialog = await screen.findByRole('dialog', { name: 'Agent detail' });
    expect(tabNames(dialog).map((name) => name.replace(/\d+$/, ''))).toEqual(labelsOf('agent'));
  });

  it('says so when the record is not on the board', async () => {
    await mountKanban({ items: [item('I-1')] }, undefined, '?open=I-99');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByText('Not found')).toBeTruthy();
  });

  it('closes with Escape and clears the address', async () => {
    await mountKanban({ items: [item('I-1')] }, undefined, '?open=I-1');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    fireEvent.keyDown(dialog, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Work item' })).toBeNull());
    expect(paneOf()).toEqual({ open: null, pane: null });
  });
});

describe('I-85 P3 Back and the old hashes', { tags: ['critical'] }, () => {
  it('goes Back to the record it came from', async () => {
    await mountKanban({ items: [item('I-1')], entries: [question] }, undefined, '?open=I-1&pane=discussion');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).queryByRole('button', { name: 'Back' })).toBeNull();
    const links = await within(dialog).findAllByRole('link', { name: 'Q-1' });
    fireEvent.click(links[0] as HTMLElement);
    const entry = await screen.findByRole('dialog', { name: 'Entry detail' });
    expect(paneOf().open).toBe('Q-1');
    fireEvent.click(within(entry).getByRole('button', { name: 'Back' }));
    await screen.findByRole('dialog', { name: 'Work item' });
    expect(paneOf().open).toBe('I-1');
  });

  it('turns an old item hash into the address and clears the hash', async () => {
    await mountKanban({ items: [item('I-1')] }, undefined, '#I-1/links');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByRole('tab', { name: /^Links/ }).getAttribute('aria-selected')).toBe('true');
    await waitFor(() => expect(paneOf()).toEqual({ open: 'I-1', pane: 'links' }));
  });

  it('opens the Work tab for the old builds tab name', async () => {
    await mountKanban({ items: [item('I-1')] }, undefined, '#I-1/builds');
    const dialog = await screen.findByRole('dialog', { name: 'Work item' });
    expect(within(dialog).getByRole('tab', { name: /^Work/ }).getAttribute('aria-selected')).toBe('true');
  });
});
