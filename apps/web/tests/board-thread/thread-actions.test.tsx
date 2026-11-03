/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { chip, cleanThreads, docs, item, mountBoard, openChip, posted, thread, tool } from './thread-harness';

cleanThreads();

const worker = (over: Record<string, unknown> = {}) => thread('worker', { brief: 'Read the plan first.', ...over });

const open = async (threadOver: Record<string, unknown> = {}, extra: Parameters<typeof mountBoard>[0] = {}) => {
  const mounted = await mountBoard({ threads: docs(worker(threadOver)), registry: docs(chip('worker')), ...extra });
  return { ...mounted, dialog: await openChip('worker') };
};

describe('S1 the owner actions of the thread drawer', { tags: ['standard'] }, () => {
  it('Pause posts a pause for the thread and tells the owner', async () => {
    const { fetchMock, dialog } = await open();
    fireEvent.click(tool(dialog, 'Pause this thread'));
    await waitFor(() => expect(posted(fetchMock)).toHaveLength(1));
    expect(posted(fetchMock)[0]).toMatchObject({ kind: 'pause', replyTo: 'worker', thread: 'worker', author: 'human', text: '' });
    expect(await screen.findByText('worker paused.')).toBeTruthy();
  });

  it('Resume posts a resume for a paused thread', async () => {
    const { fetchMock, dialog } = await open({ status: 'paused' });
    fireEvent.click(tool(dialog, 'Resume this thread'));
    await waitFor(() => expect(posted(fetchMock)).toHaveLength(1));
    expect(posted(fetchMock)[0]).toMatchObject({ kind: 'resume', replyTo: 'worker', thread: 'worker' });
  });

  it('counts a pause that is still waiting to be applied', async () => {
    const waiting = { id: 'O-5', kind: 'pause', author: 'human', replyTo: 'worker', status: 'new', createdAt: new Date().toISOString() };
    const { dialog } = await open({}, { entries: docs(['O-5', waiting]) });
    expect(within(dialog).getByRole('button', { name: 'Resume this thread' })).toBeTruthy();
  });

  it('Focus keeps the thread in the focus of the board and a second press lets it go', async () => {
    const { dialog } = await open();
    fireEvent.click(tool(dialog, 'Focus'));
    expect(localStorage.getItem('coord.focus')).toBe('worker');
    expect(tool(dialog, 'Focus').getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(tool(dialog, 'Focus'));
    expect(localStorage.getItem('coord.focus')).toBe('');
  });
});

const menu = (dialog: HTMLElement, id = 'worker'): void => {
  fireEvent.click(within(dialog).getByRole('button', { name: `More actions for ${id}` }));
};

describe('S1 the end of a thread in the thread drawer', { tags: ['standard'] }, () => {
  it('Archive sends an archive of the thread', async () => {
    const { fetchMock, dialog } = await open();
    menu(dialog);
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Archive' }));
    await waitFor(() => expect(posted(fetchMock)).toHaveLength(1));
    expect(posted(fetchMock)[0]).toMatchObject({ kind: 'archive', target: { c: 'threads', id: 'worker' } });
  });

  it('Purge opens the purge dialog for this thread', async () => {
    const { dialog } = await open();
    menu(dialog);
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Purge...' }));
    expect(await screen.findByRole('dialog', { name: /Purge/ })).toBeTruthy();
  });

  it('the lead has no Pause, Focus, Archive or Purge, only Copy id', async () => {
    await mountBoard({ threads: docs(thread('lead', { role: 'coordinator' })), registry: docs(chip('lead', { role: 'coordinator' })) });
    const dialog = await openChip('lead');
    for (const name of ['Pause this thread', 'Focus']) expect(within(dialog).queryByRole('button', { name })).toBeNull();
    menu(dialog, 'lead');
    expect(await screen.findByRole('menuitem', { name: 'Copy id' })).toBeTruthy();
    for (const name of ['Archive', 'Purge...']) expect(screen.queryByRole('menuitem', { name })).toBeNull();
  });

  it('steps to the next thread of the list with the arrows', async () => {
    await mountBoard({ threads: docs(worker(), thread('zeta')), registry: docs(chip('worker')) });
    const dialog = await openChip('worker');
    expect(within(dialog).getByRole('button', { name: /^Previous in the list/ }).hasAttribute('disabled')).toBe(true);
    fireEvent.click(within(dialog).getByRole('button', { name: /^Next in the list/ }));
    expect(await screen.findByRole('heading', { name: 'zeta session' })).toBeTruthy();
  });

  it('Expand remembers the wide drawer', async () => {
    const { dialog } = await open();
    fireEvent.click(tool(dialog, 'Expand'));
    expect(localStorage.getItem('coord.drawer.wide')).toBe('1');
    expect(tool(dialog, 'Collapse').getAttribute('aria-pressed')).toBe('true');
  });
});

describe('S1 editing in the thread drawer', { tags: ['standard'] }, () => {
  it('a new subject is sent as an edit of the thread with what it was and its revision', async () => {
    const { fetchMock, dialog } = await open();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit subject: the work' }));
    const field = within(dialog).getByRole('textbox', { name: 'Edit subject' });
    fireEvent.change(field, { target: { value: 'the new work' } });
    fireEvent.keyDown(field, { key: 'Enter' });
    await waitFor(() => expect(posted(fetchMock)).toHaveLength(1));
    expect(posted(fetchMock)[0]).toMatchObject({ kind: 'edit', target: { c: 'threads', id: 'worker' }, set: { subject: 'the new work' }, prev: { subject: 'the work' }, baseRev: 3 });
  });

  it('refuses a subject over 40 characters without a post', async () => {
    const { fetchMock, dialog } = await open();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit subject: the work' }));
    const field = within(dialog).getByRole('textbox', { name: 'Edit subject' });
    fireEvent.change(field, { target: { value: 'x'.repeat(41) } });
    fireEvent.keyDown(field, { key: 'Enter' });
    expect(await within(dialog).findByText('At most 40 characters.')).toBeTruthy();
    expect(posted(fetchMock)).toHaveLength(0);
  });

  it('the brief is sent as an edit with what it was', async () => {
    const { fetchMock, dialog } = await open();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit brief' }));
    const box = within(dialog).getByRole<HTMLTextAreaElement>('textbox', { name: 'Brief' });
    fireEvent.change(box, { target: { value: 'Read the new plan first.' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save brief' }));
    await waitFor(() => expect(posted(fetchMock)).toHaveLength(1));
    expect(posted(fetchMock)[0]).toMatchObject({ kind: 'edit', target: { c: 'threads', id: 'worker' }, set: { brief: 'Read the new plan first.' }, prev: { brief: 'Read the plan first.' } });
  });

  it('refuses a brief over 1200 characters and keeps the editor open', async () => {
    const { fetchMock, dialog } = await open();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit brief' }));
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Brief' }), { target: { value: 'x'.repeat(1201) } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save brief' }));
    expect(await within(dialog).findByText('At most 1200 characters.')).toBeTruthy();
    expect(posted(fetchMock)).toHaveLength(0);
  });
});

describe('S1 the focus kept by the drawer', { tags: ['standard'] }, () => {
  it('reaches the Kanban tab, which narrows the board to the thread', async () => {
    const { dialog } = await open({}, { items: docs(item('I-1'), item('I-2', { thread: 'other' })) });
    fireEvent.click(tool(dialog, 'Focus'));
    fireEvent.click(within(dialog).getAllByRole('button', { name: /Close/ })[0] as HTMLElement);
    fireEvent.click(await screen.findByRole('tab', { name: /^Kanban/ }));
    expect(await screen.findByRole('button', { name: /^Open K-1:/ })).toBeTruthy();
    expect(screen.queryByRole('button', { name: /^Open K-2:/ })).toBeNull();
  });
});
