/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { openOverview } from './helpers/open-overview';
import { seedProject } from './helpers/seed-project';
import { setupOverview } from './helpers/setup-overview';

setupOverview();

const LIST = 'Dependencies between projects';

const ITEM = { id: 'I-7', title: 'Wire the save hook', type: 'task', thread: 'coordinator', status: 'doing', rank: 1 };

const seedArchipelia = (view: Awaited<ReturnType<typeof openOverview>>) => seedProject(view.sources, 'archipelia', { items: { 'I-7': ITEM } });

describe('I-85 an overview row opens its pane in place', { tags: ['critical'] }, () => {
  it('opens the pane of the record on the overview page and keeps the tab under it', async () => {
    const view = await openOverview('/#dependencies');
    const list = await screen.findByRole('list', { name: LIST });
    const first = list.querySelector<HTMLElement>('[data-record="DP-1"]');
    fireEvent.click(within(first as HTMLElement).getByText('Expose the save hook'));
    await seedArchipelia(view);
    expect(await screen.findByRole('dialog')).toBeTruthy();
    expect(screen.getByRole('list', { name: LIST })).toBeTruthy();
  });

  it('opens the pane named by a board-prefixed address on the overview page', async () => {
    const view = await openOverview('/?open=archipelia%2FI-7&pane=links#dependencies');
    await seedArchipelia(view);
    expect(await screen.findByRole('dialog')).toBeTruthy();
    await waitFor(() => expect(screen.getByRole('list', { name: LIST })).toBeTruthy());
  });
});
