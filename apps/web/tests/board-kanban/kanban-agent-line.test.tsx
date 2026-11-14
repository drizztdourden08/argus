/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cardOf, cleanKanban, item, mountKanban, paneOf } from './kanban-harness';

cleanKanban();

const lineOf = (id: string): HTMLElement | null => cardOf(id).querySelector<HTMLElement>(`[data-agent-line="${id}"]`);

const working = (id: string, name: string) => ({ id, name, status: 'working' });

describe('I-85 P4 the agent line of a card', { tags: ['critical'] }, () => {
  it('names the working agent as a link with the robot mark, once', async () => {
    await mountKanban({ items: [item('I-1', { agents: [working('A-9', 'coder')] })] });
    const line = lineOf('I-1');
    expect(line?.querySelector('[data-type-mark="agent"]')).not.toBeNull();
    expect(within(line as HTMLElement).getByRole('link', { name: 'coder' })).toBeTruthy();
    expect(within(cardOf('I-1')).getAllByText('coder')).toHaveLength(1);
    expect(within(cardOf('I-1')).queryByTitle('1 agent running')).toBeNull();
  });

  it('opens the agent pane over the kanban when the name is clicked', async () => {
    const agent = { id: 'A-9', name: 'coder', thread: 'worker', task: 'do it', kind: 'build', item: 'I-1', status: 'working', rev: 1 };
    await mountKanban({ items: [item('I-1', { agents: [working('A-9', 'coder')] })], agents: [agent] });
    fireEvent.click(within(lineOf('I-1') as HTMLElement).getByRole('link', { name: 'coder' }));
    await screen.findByRole('dialog', { name: 'Agent detail' });
    expect(paneOf()).toEqual({ open: 'A-9', pane: null });
  });

  it('lists two agents as two links, each with its state', async () => {
    await mountKanban({ items: [item('I-1', { agents: [working('A-9', 'coder'), working('A-10', 'checker')] })] });
    const line = within(lineOf('I-1') as HTMLElement);
    expect(line.getByRole('link', { name: 'coder' })).toBeTruthy();
    expect(line.getByRole('link', { name: 'checker' })).toBeTruthy();
    expect(line.getAllByText('working')).toHaveLength(2);
  });

  it('draws no line for an item nobody works on', async () => {
    await mountKanban({ items: [item('I-1'), item('I-2', { agents: [] })] });
    expect(lineOf('I-1')).toBeNull();
    expect(lineOf('I-2')).toBeNull();
  });

  it('starts the line of row four, before the chips and the stage', async () => {
    await mountKanban({ items: [item('I-1', { type: 'bug', stage: 'fixing', status: 'doing', fields: { severity: 'S2' }, agents: [working('A-9', 'coder')] })] });
    const line = lineOf('I-1') as HTMLElement;
    const stage = within(cardOf('I-1')).getByText('Fixing');
    expect(Boolean(line.compareDocumentPosition(stage) & Node.DOCUMENT_POSITION_FOLLOWING)).toBe(true);
  });
});
