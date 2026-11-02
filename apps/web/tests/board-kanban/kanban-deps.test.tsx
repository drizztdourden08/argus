/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, ago, boardOf, cleanKanban, item, mountKanban, openItem } from './kanban-harness';

cleanKanban();

const waitsOn = (extra: Record<string, unknown> = {}) => ({
  dep: 'D-4', project: 'rotp', item: 'I-12', title: 'Expose the save hook', state: 'open', since: ago(2 * HOUR), reason: null, ...extra,
});

const requestedBy = { dep: 'D-5', project: 'archipelia', item: 'I-7', thread: 'T-2', title: 'Save the game', state: 'open' };

const titleOf = (text: string): string | null => screen.getByText(text).closest('[title]')?.getAttribute('title') ?? null;

describe('S1 the dependency tags on the cards', { tags: ['standard'] }, () => {
  it('shows one tag per state and side, with the other title and the age as the tooltip', async () => {
    await mountKanban({
      items: [
        item('I-1', { waitsOn: waitsOn() }),
        item('I-2', { waitsOn: waitsOn({ state: 'blocked', reason: 'needs the schema' }) }),
        item('I-3', { waitsOn: waitsOn({ state: 'done' }) }),
        item('I-4', { requestedBy }),
        item('I-5', { waitsOn: waitsOn({ state: 'cancelled' }) }),
      ],
    });
    const board = within(await boardOf());
    expect(board.getByText('waiting on rotp/I-12')).toBeTruthy();
    expect(board.getByText('rotp/I-12 blocked')).toBeTruthy();
    expect(board.getByText('rotp/I-12 done')).toBeTruthy();
    expect(board.getByText('for archipelia/I-7')).toBeTruthy();
    expect(board.getByText('request cancelled')).toBeTruthy();
    expect(titleOf('waiting on rotp/I-12')).toBe('Expose the save hook. 2h ago');
    expect(titleOf('rotp/I-12 blocked')).toBe('Expose the save hook. Reason: needs the schema. 2h ago');
    expect(titleOf('for archipelia/I-7')).toMatch(/^Save the game\./);
  });

  it('shows no dependency tag on an item with no dependency', async () => {
    await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item I-1');
    expect(screen.queryByText(/waiting on|request cancelled|^for /)).toBeNull();
  });
});

describe('S1 the Dependency section of the drawer', { tags: ['standard'] }, () => {
  const blocked = () => item('I-1', { waitsOn: waitsOn({ state: 'blocked', reason: 'needs the schema' }) });

  it('shows the state, the other item as a link to its board, the reason and the age', async () => {
    await mountKanban({ items: [blocked()] });
    const dialog = await openItem('I-1');
    expect(within(dialog).getByText('Dependency')).toBeTruthy();
    expect(within(dialog).getAllByText('blocked').length).toBeGreaterThan(0);
    expect(within(dialog).getByRole('link', { name: 'rotp/I-12 Expose the save hook' }).getAttribute('href')).toBe('/p/rotp#I-12');
    expect(within(dialog).getByText('Reason: needs the schema')).toBeTruthy();
    expect(within(dialog).getByText('Open for 2h')).toBeTruthy();
  });

  it('has no Dependency section on an item with no dependency', async () => {
    await mountKanban({ items: [item('I-1')] });
    const dialog = await openItem('I-1');
    expect(within(dialog).queryByText('Dependency')).toBeNull();
  });

  it('cancels as the owner and shows the answer text as a toast', async () => {
    const { posted } = await mountKanban({ items: [blocked()] }, (body) => (body.actor === 'owner' ? { dep: {}, text: 'D-4 is cancelled.' } : undefined));
    const dialog = await openItem('I-1');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(posted.at(-1)).toMatchObject({ url: '/api/deps/D-4/cancel', body: { actor: 'owner' } }));
    expect(await screen.findByText('D-4 is cancelled.')).toBeTruthy();
  });

  it('offers no Cancel once the request is done', async () => {
    await mountKanban({ items: [item('I-1', { waitsOn: waitsOn({ state: 'done' }) })] });
    const dialog = await openItem('I-1');
    expect(within(dialog).queryByRole('button', { name: 'Cancel' })).toBeNull();
  });
});
