/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ATTENTION } from '../../src/screens/board/tabs/kanban/attention';
import { MIN, ago, boardOf, cleanKanban, item, mountKanban } from './kanban-harness';

cleanKanban();

const cardOf = (id: string): HTMLElement => {
  const card = document.querySelector(`[data-item="${id}"]`)?.closest('.kanban__card');
  if (!(card instanceof HTMLElement)) throw new Error(`no card for ${id}`);
  return card;
};

const inOrder = (first: HTMLElement, second: HTMLElement): boolean =>
  Boolean(first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING);

const chipOf = (card: HTMLElement, title: string): HTMLElement => within(card).getByTitle(title);

describe('I-69 the signal strip on a card', { tags: ['standard'] }, () => {
  it('shows an icon and a count for each kind in precedence order, and the lead kind colours the edge', async () => {
    await mountKanban({
      items: [item('I-1'), item('I-2'), item('I-3')],
      entries: [{ id: 'D-1', kind: 'decision', item: 'I-1', author: 'worker', status: 'pending', createdAt: ago(MIN) }],
      tests: [
        { id: 'T-1', item: 'I-1', mode: 'auto', status: 'fail', title: 'a', ac: [] },
        { id: 'T-2', item: 'I-2', mode: 'auto', status: 'pending', title: 'b', ac: [] },
      ],
      agents: [{ id: 'A-1', item: 'I-1', status: 'working', name: 'coder' }, { id: 'A-2', item: 'I-3', status: 'working', name: 'coder' }],
    });
    await boardOf();
    const first = cardOf('I-1');
    const failing = chipOf(first, '1 test failing');
    const decision = chipOf(first, 'D-1 is a decision to pick up');
    const agent = chipOf(first, '1 agent running');
    expect(inOrder(failing, decision)).toBe(true);
    expect(inOrder(decision, agent)).toBe(true);
    expect(failing.querySelector('svg')).toBeTruthy();
    expect(failing.textContent).toBe('1');
    expect(cardOf('I-1').className).toContain('card--edge-danger');
    expect(chipOf(cardOf('I-2'), '1 test to run')).toBeTruthy();
    expect(cardOf('I-2').className).toContain('card--edge-info');
    expect(cardOf('I-3').className).toContain('card--edge-success');
  });

  it('draws no edge and no strip on a card with nothing to say', async () => {
    await mountKanban({ items: [item('I-1')] });
    await boardOf();
    expect(cardOf('I-1').className).not.toContain('card--edge-');
    expect(within(cardOf('I-1')).queryByRole('group', { name: 'Signals' })).toBeNull();
  });

  it('folds what does not fit into a +N chip', async () => {
    const blocked = [
      { kind: 'dep-waits', id: 'D-4', label: 'Expose the hook', who: 'lead:rotp' },
      { kind: 'todo', id: 'H-12', label: 'push go', who: 'owner' },
    ];
    await mountKanban({
      items: [item('I-1', { status: 'doing', blocked })],
      entries: [
        { id: 'Q-1', kind: 'question', item: 'I-1', author: 'worker', status: 'open', createdAt: ago(MIN) },
        { id: 'D-1', kind: 'decision', item: 'I-1', author: 'worker', status: 'pending', createdAt: ago(MIN) },
      ],
      actions: [{ id: 'A-1', item: 'I-1', status: 'pending' }],
      tests: [
        { id: 'T-1', item: 'I-1', mode: 'auto', status: 'fail', title: 'a', ac: [] },
        { id: 'T-2', item: 'I-1', mode: 'auto', status: 'pending', title: 'b', ac: [] },
      ],
      agents: [{ id: 'A-1', item: 'I-1', status: 'working', name: 'coder' }],
    });
    await boardOf();
    const strip = within(cardOf('I-1')).getByRole('group', { name: 'Signals' });
    expect(within(strip).getByText('+3')).toBeTruthy();
    expect(within(strip).queryByTitle('1 agent running')).toBeNull();
  });

  it('opens the item from a dependency chip', async () => {
    const blocked = [{ kind: 'dep-waits', id: 'D-4', label: 'Expose the hook', who: 'lead:rotp' }];
    await mountKanban({
      items: [item('I-1', { blocked })],
      entries: [{ id: 'Q-1', kind: 'question', item: 'I-1', author: 'worker', status: 'open', createdAt: ago(MIN) }],
    });
    await boardOf();
    fireEvent.click(chipOf(cardOf('I-1'), 'waiting on rotp: Expose the hook'));
    expect(await screen.findByRole('dialog', { name: 'Work item' })).toBeTruthy();
  });
});

describe('I-72 the head and the foot of a card', { tags: ['standard'] }, () => {
  it('shows the type mark, the id and the assignee on one row, and leaves the subject and the thread out', async () => {
    await mountKanban({ items: [item('I-1', { type: 'bug', subject: 'billing', assignee: 'owner', stage: 'fixing', status: 'doing', fields: { severity: 'S2' } })] });
    await boardOf();
    const card = within(cardOf('I-1'));
    expect(card.getByRole('img', { name: 'Bug' })).toBeTruthy();
    expect(card.getByRole('img', { name: 'Assigned to you' })).toBeTruthy();
    expect(card.getByText('I-1')).toBeTruthy();
    expect(card.getByText('S2')).toBeTruthy();
    expect(card.getByText('Fixing')).toBeTruthy();
    expect(card.queryByText('billing')).toBeNull();
    expect(card.queryByText('worker')).toBeNull();
  });
});

describe('I-69 the legend under the toolbar', { tags: ['standard'] }, () => {
  it('is folded and lists every type mark and every kind with its icon', async () => {
    await mountKanban({ items: [item('I-1')] });
    await boardOf();
    const legend = screen.getByText('Legend').closest('details');
    expect(legend).not.toBeNull();
    expect(legend?.open).toBe(false);
    const rows = within(legend as HTMLElement);
    for (const type of ['Epic', 'Story', 'Task', 'Bug', 'Tech debt', 'Test plan']) expect(rows.getByRole('img', { name: type, hidden: true })).toBeTruthy();
    for (const spec of ATTENTION) {
      const tag = rows.getByText(spec.label);
      expect(tag.parentElement?.querySelector('svg')).not.toBeNull();
      expect(rows.getByText(spec.text)).toBeTruthy();
    }
  });
});
