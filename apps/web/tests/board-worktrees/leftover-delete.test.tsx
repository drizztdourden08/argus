/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ago, leftover, metaDoc, oldMeta } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import type { Responder } from './mount-tab';
import { chooseAction, confirmGroup, text } from './wt-dom';

cleanAfterEach();

const LEFTOVER_GROUP = 'Confirm deleting the leftover folder';
const BATCH_GROUP = 'Confirm deleting the old leftover folders';
const DELETE_LEFTOVER = /^Delete leftover/;

const started = (body: object): Responder => () => ({ status: 200, body });

const refuse = (code: string, message: string): Responder => () => ({ status: 409, body: { error: { code, message } } });

const deleteButton = (name: string): HTMLButtonElement => within(confirmGroup(LEFTOVER_GROUP)).getByRole<HTMLButtonElement>('button', { name });

describe('I-85 Delete leftover from the row menu: the confirm', { tags: ['standard'] }, () => {
  it('opens a confirm with the size, the files, the newest change and that no backup is possible', async () => {
    await mountTab({ docs: [leftover('agent-ab12'), leftover('other')], meta: oldMeta() });
    await chooseAction('agent-ab12', DELETE_LEFTOVER);
    const words = text(confirmGroup(LEFTOVER_GROUP));
    expect(words).toContain('Delete the leftover folder X:/proj/trees/agent-ab12?');
    expect(words).toContain('It holds about 1.5 GB in 34210 files');
    expect(words).toContain('its newest change was');
    expect(words).toContain('No backup is possible: git does not know this folder.');
    expect(words).toContain('Argus renames it and deletes it in the background.');
    expect(words).not.toContain('changed less than');
    expect(deleteButton('Delete leftover')).toBeTruthy();
  });

  it('a folder younger than the limit gets the stronger wording and Delete anyway; the route is told it was confirmed', async () => {
    const young = leftover('fresh-1', { ageDays: 0 }, { ageDays: 0, newestAt: ago(5 * 3_600_000) });
    const { calls } = await mountTab({ docs: [young], meta: oldMeta() }, started({ started: true, jobId: 'L-1' }));
    await chooseAction('fresh-1', DELETE_LEFTOVER);
    expect(text(confirmGroup(LEFTOVER_GROUP))).toContain('It changed less than 3 days ago: something may still be writing there. A thread would refuse it; confirm only if you are sure it is dead.');
    fireEvent.click(deleteButton('Delete anyway'));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toEqual({ id: 'id-fresh-1', newestAt: young.leftover?.newestAt, confirmYoung: true });
  });
});

describe('I-85 Delete leftover: the answer', { tags: ['standard'] }, () => {
  it('Confirm posts the route with the newest change the owner saw; the answer names the job', async () => {
    const row = leftover('agent-ab12');
    const { calls } = await mountTab({ docs: [row], meta: oldMeta() }, started({ started: true, jobId: 'L-7' }));
    await chooseAction('agent-ab12', DELETE_LEFTOVER);
    fireEvent.click(deleteButton('Delete leftover'));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/alpha/worktrees/remove', method: 'POST' });
    expect(calls[0]?.body).toEqual({ id: 'id-agent-ab12', newestAt: row.leftover?.newestAt, confirmYoung: false });
    expect(await screen.findByText('Deleting agent-ab12 in the background (job L-7); the row goes when the folder is gone.')).toBeTruthy();
    expect(screen.queryByRole('group', { name: LEFTOVER_GROUP })).toBeNull();
  });

  it('a refusal stays in the confirm with the server sentence, and it can be tried again', async () => {
    await mountTab({ docs: [leftover('agent-ab12')], meta: oldMeta() }, refuse('conflict', 'X:/p/agent-ab12 changed since you looked <now>'));
    await chooseAction('agent-ab12', DELETE_LEFTOVER);
    fireEvent.click(deleteButton('Delete leftover'));
    await waitFor(() => expect(text(confirmGroup(LEFTOVER_GROUP))).toContain('Not removed: X:/p/agent-ab12 changed since you looked <now>'));
    expect(deleteButton('Delete leftover').disabled).toBe(false);
    fireEvent.click(within(confirmGroup(LEFTOVER_GROUP)).getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('group', { name: LEFTOVER_GROUP })).toBeNull();
  });

  it('blocks Confirm when something was written in the folder after the confirm opened', async () => {
    const { calls, change } = await mountTab({ docs: [leftover('agent-ab12')], meta: oldMeta() });
    await chooseAction('agent-ab12', DELETE_LEFTOVER);
    change('worktrees', 'id-agent-ab12', { ...leftover('agent-ab12', {}, { newestAt: ago(60_000), ageDays: 0 }) });
    await waitFor(() => expect(text(confirmGroup(LEFTOVER_GROUP))).toContain('This changed since you opened this confirm.'));
    expect(deleteButton('Delete leftover').disabled).toBe(true);
    expect(calls).toEqual([]);
  });
});

describe('I-85 Delete all leftovers: the confirm and the answer', { tags: ['standard'] }, () => {
  const button = 'Delete all leftovers older than 3 days (2, about 2 GB)';

  it('opens a confirm with the count, the total size and the re-checks; Cancel closes it and posts nothing', async () => {
    const { calls } = await mountTab({ docs: [leftover('a'), leftover('b')], meta: oldMeta() });
    fireEvent.click(await screen.findByRole('button', { name: button }));
    const words = text(confirmGroup(BATCH_GROUP));
    expect(words).toContain('Delete 2 leftover folders older than 3 days?');
    expect(words).toContain('Together about 2 GB (node_modules is not counted).');
    expect(words).toContain('No backup is possible: git does not know these folders.');
    expect(within(confirmGroup(BATCH_GROUP)).getByRole('button', { name: 'Delete 2 folders' })).toBeTruthy();
    fireEvent.click(within(confirmGroup(BATCH_GROUP)).getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('group', { name: BATCH_GROUP })).toBeNull();
    expect(calls).toEqual([]);
  });

  it('Confirm posts the batch route; the answer counts the queued folders and the skipped ones with their reasons', async () => {
    const skipped = [{ id: 'x', reason: 'holds a nested git repository (inner/.git)' }, { id: 'y', reason: 'holds a nested git repository (inner/.git)' }];
    const { calls } = await mountTab({ docs: [leftover('a'), leftover('b')], meta: oldMeta() }, started({ started: 1, skipped, jobs: [] }));
    fireEvent.click(await screen.findByRole('button', { name: button }));
    fireEvent.click(within(confirmGroup(BATCH_GROUP)).getByRole('button', { name: 'Delete 2 folders' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/alpha/worktrees/leftovers/remove', method: 'POST' });
    expect(calls[0]?.body).toEqual({ olderThanDays: 3 });
    const sentence = 'Queued 1 leftover folder for deletion, skipped 2 (holds a nested git repository (inner/.git)). Argus deletes them one after another in the background; each row goes when its folder is gone.';
    expect(await screen.findByText(sentence)).toBeTruthy();
    expect(screen.queryByRole('group', { name: BATCH_GROUP })).toBeNull();
  });

  it('a refusal stays in the confirm and can be retried; under 1 MB says so', async () => {
    await mountTab({ docs: [leftover('a')], meta: metaDoc({ leftoverOld: { days: 3, count: 1, sizeMB: 0.2 } }) }, refuse('forbidden', 'only the lead may'));
    fireEvent.click(await screen.findByRole('button', { name: 'Delete all leftovers older than 3 days (1, under 1 MB)' }));
    fireEvent.click(within(confirmGroup(BATCH_GROUP)).getByRole('button', { name: 'Delete 1 folder' }));
    await waitFor(() => expect(text(confirmGroup(BATCH_GROUP))).toContain('Not queued: only the lead may'));
    expect(within(confirmGroup(BATCH_GROUP)).getByRole<HTMLButtonElement>('button', { name: 'Delete 1 folder' }).disabled).toBe(false);
  });
});
