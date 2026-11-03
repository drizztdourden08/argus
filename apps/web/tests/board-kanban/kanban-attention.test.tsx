/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
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

describe('I-69 the attention tags on a card', { tags: ['standard'] }, () => {
  it('shows an icon tag for each kind in precedence order, and the lead kind colours the edge', async () => {
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
    const first = within(cardOf('I-1'));
    const failing = first.getByText('1 failing');
    const decision = first.getByText('D-1 to pick up');
    const agent = first.getByText('agent running');
    expect(inOrder(failing, decision)).toBe(true);
    expect(inOrder(decision, agent)).toBe(true);
    expect(failing.parentElement?.querySelector('svg')).toBeTruthy();
    expect(cardOf('I-1').className).toContain('card--edge-danger');
    expect(within(cardOf('I-2')).getByText('1 to run')).toBeTruthy();
    expect(cardOf('I-2').className).toContain('card--edge-info');
    expect(cardOf('I-3').className).toContain('card--edge-success');
  });

  it('draws no edge on a card with nothing to say', async () => {
    await mountKanban({ items: [item('I-1')] });
    await boardOf();
    expect(cardOf('I-1').className).not.toContain('card--edge-');
  });
});

describe('I-69 the legend under the toolbar', { tags: ['standard'] }, () => {
  it('is folded and lists every kind with its icon', async () => {
    await mountKanban({ items: [item('I-1')] });
    await boardOf();
    const legend = screen.getByText('Legend').closest('details');
    expect(legend).not.toBeNull();
    expect(legend?.open).toBe(false);
    const rows = within(legend as HTMLElement);
    for (const spec of ATTENTION) {
      const tag = rows.getByText(spec.label);
      expect(tag.parentElement?.querySelector('svg')).not.toBeNull();
      expect(rows.getByText(spec.text)).toBeTruthy();
    }
  });
});
