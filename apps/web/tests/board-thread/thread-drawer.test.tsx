/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { chip, cleanThreads, clickChip, docs, drawerOf, item, mountBoard, openChip, thread } from './thread-harness';

cleanThreads();

const worker = (over: Record<string, unknown> = {}) => thread('worker', { brief: 'Brief with `code`', ...over });

describe('S1 opening the thread drawer', { tags: ['standard'] }, () => {
  it('a chip of the registered strip opens it, with the note and the brief rendered as markdown', async () => {
    await mountBoard({ threads: docs(worker()), registry: docs(chip('worker')) });
    const dialog = within(await openChip('worker'));
    expect(dialog.getByRole('heading', { name: 'worker session' })).toBeTruthy();
    const now = dialog.getByRole('group', { name: 'Now' });
    expect(within(now).getByText('the thing').tagName).toBe('STRONG');
    const brief = dialog.getByRole('group', { name: 'Brief' });
    expect(within(brief).getByText('code').tagName).toBe('CODE');
  });

  it('a chip of the lead opens it too, once the worker drawer is closed', async () => {
    await mountBoard({ threads: docs(worker(), thread('lead', { role: 'lead' })), registry: docs(chip('worker'), chip('lead', { role: 'lead' })) });
    expect(within(await openChip('worker')).getByRole('heading', { name: 'worker session' })).toBeTruthy();
    fireEvent.click(within(await drawerOf()).getAllByRole('button', { name: /Close/ })[0] as HTMLElement);
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Thread detail' })).toBeNull());
    expect(within(await openChip('lead')).getByRole('heading', { name: 'lead session' })).toBeTruthy();
  });
});

describe('S1 what the thread drawer shows', { tags: ['standard'] }, () => {
  it('counts only the open questions and the pending decisions', async () => {
    const entry = (id: string, over: Record<string, unknown>) => [id, { id, thread: 'worker', author: 'worker', createdAt: new Date().toISOString(), ...over }] as const;
    await mountBoard({
      threads: docs(worker()),
      registry: docs(chip('worker')),
      entries: docs(
        ...[
          entry('Q-1', { kind: 'question', status: 'open', text: 'One?' }),
          entry('Q-2', { kind: 'question', status: 'answered', text: 'Two?' }),
          entry('D-1', { kind: 'decision', status: 'pending', text: 'Do it' }),
          entry('D-2', { kind: 'decision', status: 'applied', text: 'Done' }),
        ].map(([id, doc]) => [id, doc] as [string, object]),
      ),
    });
    const dialog = within(await openChip('worker'));
    expect(dialog.getByText('1 open')).toBeTruthy();
    expect(dialog.getByText('1 to pick up')).toBeTruthy();
  });
});

describe('S1 the open work in the thread drawer', { tags: ['standard'] }, () => {
  it('lists the open work of the thread only, and opens the item in its pane', async () => {
    await mountBoard({
      threads: docs(worker()),
      registry: docs(chip('worker')),
      items: docs(item('I-1'), item('I-2', { status: 'done' }), item('I-3', { thread: 'other' }), item('I-4', { status: 'dropped' }), item('I-5', { status: 'todo' })),
    });
    const dialog = within(await openChip('worker'));
    const work = within(dialog.getByRole('group', { name: 'Open work' }));
    expect(work.getAllByRole('button').map((button) => button.getAttribute('aria-label'))).toEqual(['Open K-1', 'Open K-5']);
    fireEvent.click(work.getByRole('button', { name: 'Open K-5' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Thread detail' })).toBeNull());
    expect(await screen.findByRole('dialog', { name: 'Work item' })).toBeTruthy();
  });

  it('shows 15 items and says how many more there are', async () => {
    const many = Array.from({ length: 17 }, (_, index) => item(`I-${index + 1}`, { rank: index }));
    await mountBoard({ threads: docs(worker()), registry: docs(chip('worker')), items: docs(...many) });
    const work = within((await within(await openChip('worker')).findByRole('group', { name: 'Open work' })));
    expect(work.getAllByRole('button')).toHaveLength(15);
    expect(work.getByText('and 2 more: filter the board by this thread.')).toBeTruthy();
  });

  it('does not open for a thread the board has dropped', async () => {
    await mountBoard({ threads: docs(worker({ deletedAt: new Date().toISOString() })), registry: docs(chip('worker')) });
    await clickChip('worker');
    expect(screen.queryByRole('dialog', { name: 'Thread detail' })).toBeNull();
  });
});
