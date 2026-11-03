/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanAfterEach } from '../board-tests/board-dom';
import { groupOf, order } from '../helpers/group-dom';
import { ago, DAY, launcherDoc } from './fixtures';
import { withDefs } from './list-dom';
import { mountBuilds } from './mount-builds';

cleanAfterEach();

const itemDoc = (id: string, extra: object = {}) => ({
  id, title: `Item ${id}`, type: 'task', stage: 'doing', status: 'doing', thread: 'lead', rank: 1024, ...extra,
});

const board = () => ({
  ...withDefs([
    launcherDoc('master', { master: true }),
    launcherDoc('web', { for: 'I-2', stage: 'ready' }),
    launcherDoc('loose'),
    launcherDoc('guess', { for: 'I-2', relink: true }),
    launcherDoc('gone', { for: 'I-3', retiredAt: ago(DAY), retiredByItem: 'I-3', stage: 'closed' }),
  ]),
  items: [itemDoc('I-2'), itemDoc('I-3', { status: 'done', stage: 'done' })],
});

describe('P13 the Builds tab by item', { tags: ['standard'] }, () => {
  it('shows the master build, then each open item, then the builds that need a link', async () => {
    const { container } = await mountBuilds(board());
    await screen.findByText('Master build');
    expect(order(container)).toEqual(['master', 'I-2', 'none']);
    const item = groupOf(container, 'I-2');
    expect(within(item).getByText('Item I-2')).toBeTruthy();
    expect(item.querySelector('[data-launcher="web"]')).not.toBeNull();
    const needs = groupOf(container, 'none');
    expect(within(needs).getByText('Needs a link')).toBeTruthy();
    expect(needs.querySelector('[data-launcher="loose"]')).not.toBeNull();
    expect(needs.querySelector('[data-launcher="guess"]')).not.toBeNull();
  });

  it('keeps the builds closed with their item in a collapsed Closed section that names the item, without Play', async () => {
    const { container } = await mountBuilds(board());
    const toggle = await screen.findByText('Closed (1)');
    const details = toggle.closest('details') as HTMLDetailsElement;
    expect(details.open).toBe(false);
    fireEvent.click(toggle.closest('summary') as HTMLElement);
    const card = container.querySelector('[data-launcher="gone"]') as HTMLElement;
    expect(card.textContent).toContain('closed with I-3');
    expect(within(card).queryByRole('button', { name: /play/i })).toBeNull();
  });

  it('gives the master build no link and a card that opens its pane', async () => {
    const { container } = await mountBuilds(board());
    await screen.findByText('Master build');
    const master = container.querySelector('[data-launcher="master"]') as HTMLElement;
    expect(within(master).getByRole('button', { name: /^Open BL-master/ })).toBeTruthy();
    expect(within(master).queryByText(/Serves no item/)).toBeNull();
  });
});
