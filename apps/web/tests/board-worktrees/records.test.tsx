/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { itemRef } from '../board-kanban/kanban-docs';
import { groupOf } from '../helpers/group-dom';
import { HOUR, ago } from './fixtures';
import { cleanAfterEach, findRow, metaDoc, mountTab, text, worktree } from './wt-kit';

cleanAfterEach();

const request = (extra: object = {}) => ({ id: 'R-3', by: 'lead', at: ago(HOUR), mode: 'released', status: 'done', total: 2, removed: 2, refused: 0, ...extra });

const gone = { id: 'old', path: 'X:/proj/trees/gone', thread: 'lead', at: ago(2 * HOUR), text: 'Removed gone' };

describe('I-85 the title of a worktree row', { tags: ['standard'] }, () => {
  it('says the task of the agent that owns the worktree', async () => {
    const agents = [{ id: 'A-1', name: 'builder', task: 'Build the pane', status: 'running' }];
    await mountTab({ meta: metaDoc(), agents, docs: [worktree('a', 'active', { agent: 'A-1' })] });
    expect(within(await findRow('a')).getByText('Build the pane')).toBeTruthy();
  });

  it('says the branch when it differs from the folder name', async () => {
    await mountTab({ meta: metaDoc(), docs: [worktree('b', 'active', { branch: 'feat/b' })] });
    expect(text(await findRow('b'))).toContain('feat/b');
  });

  it('says the folder under the rule when the branch only repeats the folder name', async () => {
    const row = worktree('agent-c', 'active', { branch: 'worktree-agent-c', path: 'X:/proj/trees/sub/agent-c' });
    await mountTab({ meta: metaDoc(), docs: [row] });
    const words = text(await findRow('agent-c'));
    expect(words).toContain('sub/agent-c');
    expect(words).not.toContain('worktree-agent-c');
  });
});

describe('I-85 the group header of an item', { tags: ['standard'] }, () => {
  it('spans the width with the type mark, the code link, the title and the count', async () => {
    const { container } = await mountTab({ meta: metaDoc(), items: [itemRef('I-1')], docs: [worktree('linked', 'active', { item: 'I-1' })] });
    await screen.findByText('Item number 1');
    const group = groupOf(container, 'I-1');
    expect(group.querySelector('summary [data-type-mark]')).not.toBeNull();
    expect(within(group).getByRole('link', { name: /-1/ })).toBeTruthy();
    expect(group.querySelector('summary [data-record]')).toBeNull();
    expect(group.querySelector('.disclosure__end')?.textContent).toBe('1');
  });
});

describe('I-85 the requests and the removed worktrees as rows', { tags: ['standard'] }, () => {
  const board = () => ({ docs: [worktree('live', 'active')], meta: metaDoc({ requests: [request()], hygiene: { on: true, graceMs: 0, removed: [gone] } }) });

  it('folds both lists by default under a count', async () => {
    const { container } = await mountTab(board());
    const requests = (await screen.findByText('Removal requests from threads')).closest('details');
    const removed = screen.getByText('Removed by the worktree hygiene').closest('details');
    expect(requests?.open).toBe(false);
    expect(removed?.open).toBe(false);
    expect(requests?.querySelector('.disclosure__end')?.textContent).toBe('1');
    expect(removed?.querySelector('.disclosure__end')?.textContent).toBe('1');
    expect(container.querySelector('[data-worktree="id-live"]')).not.toBeNull();
  });

  it('shows a request as a row with its code, status chip and age', async () => {
    const { container } = await mountTab(board());
    await screen.findByText('Removal requests from threads');
    const row = container.querySelector<HTMLElement>('[data-record="R-3"]');
    expect(row).not.toBeNull();
    expect(text(row as HTMLElement)).toContain('lead asked to remove its 2 worktrees released after the grace');
    expect(within(row as HTMLElement).getByText('done')).toBeTruthy();
    expect(within(row as HTMLElement).getByText('2 removed')).toBeTruthy();
    expect(within(row as HTMLElement).getByText('1h')).toBeTruthy();
  });

  it('shows a removed worktree as a WT row with a removed chip and its age', async () => {
    const { container } = await mountTab(board());
    await screen.findByText('Removed by the worktree hygiene');
    const row = container.querySelector<HTMLElement>('[data-record="WT gone"]');
    expect(row).not.toBeNull();
    expect(within(row as HTMLElement).getByText('removed')).toBeTruthy();
    expect(within(row as HTMLElement).getByText('X:/proj/trees/gone')).toBeTruthy();
    expect(within(row as HTMLElement).getByText('2h')).toBeTruthy();
  });
});
