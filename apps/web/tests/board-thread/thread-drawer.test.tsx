/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { chip, cleanThreads, docs, drawerOf, item, mountBoard, openChip, thread } from './thread-harness';

cleanThreads();

const worker = (over: Record<string, unknown> = {}) => thread('worker', { brief: 'Brief with `code`', ...over });

describe('opening the thread drawer', () => {
  it('a chip of the registered strip opens it with the session title, the id, the note and the brief', async () => {
    await mountBoard({ threads: docs(worker()), registry: docs(chip('worker')) });
    const dialog = within(await openChip('worker'));
    expect(dialog.getByRole('heading', { name: 'worker session' })).toBeTruthy();
    expect(dialog.getByText('worker', { selector: '[class*="subtitle"], p, span' })).toBeTruthy();
    const now = dialog.getByRole('group', { name: 'Now' });
    expect(within(now).getByText('the thing').tagName).toBe('STRONG');
    const brief = dialog.getByRole('group', { name: 'Brief' });
    expect(within(brief).getByText('code').tagName).toBe('CODE');
  });

  it('a chat of the Agents list opens it, and so does the note of the lead', async () => {
    await mountBoard({ threads: docs(worker(), thread('lead', { role: 'coordinator', note: 'Planning the week' })) });
    fireEvent.click(await screen.findByText('worker', { selector: '[data-chat-id] *' }));
    expect(within(await drawerOf()).getByRole('heading', { name: 'worker session' })).toBeTruthy();
    fireEvent.click(within(await drawerOf()).getAllByRole('button', { name: /Close/ })[0] as HTMLElement);
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Thread detail' })).toBeNull());
    fireEvent.click(await screen.findByRole('button', { name: 'Planning the week' }));
    const dialog = within(await drawerOf());
    expect(dialog.getByRole('heading', { name: 'lead session' })).toBeTruthy();
    expect(dialog.getByText('coord')).toBeTruthy();
  });

  it('says there is no note, and shows no Brief section for a thread without a brief', async () => {
    await mountBoard({ threads: docs(thread('worker', { note: '' })), registry: docs(chip('worker')) });
    const dialog = within(await openChip('worker'));
    expect(dialog.getByText('No note.')).toBeTruthy();
    expect(dialog.queryByRole('group', { name: 'Brief' })).toBeNull();
  });
});

describe('what the thread drawer shows', () => {
  it('has the status, the activity, the subject, the age and the open questions and decisions', async () => {
    const entry = (id: string, over: Record<string, unknown>) => [id, { id, thread: 'worker', author: 'worker', createdAt: new Date().toISOString(), ...over }] as const;
    await mountBoard({
      threads: docs(worker({ root: 'X:/work/worker' })),
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
    expect(dialog.getByText('active')).toBeTruthy();
    expect(dialog.getByText('working')).toBeTruthy();
    expect(dialog.getByRole('button', { name: /the work/ })).toBeTruthy();
    expect(dialog.getByText('1 open')).toBeTruthy();
    expect(dialog.getByText('1 to pick up')).toBeTruthy();
    expect(dialog.getByText('X:/work/worker')).toBeTruthy();
    expect(dialog.getByText(/seen/)).toBeTruthy();
  });

  it('lists the agents of the thread with their task', async () => {
    const agents = [{ name: 'build', task: 'Port the **tabs**', startedAt: new Date().toISOString() }];
    await mountBoard({ threads: docs(worker({ agents })), registry: docs(chip('worker')) });
    const section = within((await within(await openChip('worker')).findByRole('group', { name: 'Agents' })));
    expect(section.getByText('build')).toBeTruthy();
    expect(section.getByText('Port the **tabs**')).toBeTruthy();
  });

});

describe('the open work in the thread drawer', () => {
  it('lists the open work of the thread only, and opens the item on the Kanban tab', async () => {
    await mountBoard({
      threads: docs(worker()),
      registry: docs(chip('worker')),
      items: docs(item('I-1'), item('I-2', { status: 'done' }), item('I-3', { thread: 'other' }), item('I-4', { status: 'dropped' }), item('I-5', { status: 'todo' })),
    });
    const dialog = within(await openChip('worker'));
    const work = within(dialog.getByRole('group', { name: 'Open work' }));
    expect(work.getAllByRole('button').map((button) => button.getAttribute('aria-label'))).toEqual(['Open I-1', 'Open I-5']);
    fireEvent.click(work.getByRole('button', { name: 'Open I-5' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Thread detail' })).toBeNull());
    expect(await screen.findByRole('tab', { name: /^Kanban/ })).toBeTruthy();
    await waitFor(() => expect(screen.getByRole('tab', { name: /^Kanban/ }).getAttribute('aria-selected')).toBe('true'));
  });

  it('shows 15 items and says how many more there are', async () => {
    const many = Array.from({ length: 17 }, (_, index) => item(`I-${index + 1}`, { rank: index }));
    await mountBoard({ threads: docs(worker()), registry: docs(chip('worker')), items: docs(...many) });
    const work = within((await within(await openChip('worker')).findByRole('group', { name: 'Open work' })));
    expect(work.getAllByRole('button')).toHaveLength(15);
    expect(work.getByText('and 2 more: filter the board by this thread.')).toBeTruthy();
  });

  it('says so when the thread has no open work', async () => {
    await mountBoard({ threads: docs(worker()), registry: docs(chip('worker')) });
    expect(within(await openChip('worker')).getByText('No open work item.')).toBeTruthy();
  });

  it('does not open for a thread the board has dropped', async () => {
    await mountBoard({ threads: docs(worker({ deletedAt: new Date().toISOString() })), registry: docs(chip('worker')) });
    fireEvent.click(await screen.findByRole('button', { name: 'Open the thread worker' }));
    expect(screen.queryByRole('dialog', { name: 'Thread detail' })).toBeNull();
  });
});
