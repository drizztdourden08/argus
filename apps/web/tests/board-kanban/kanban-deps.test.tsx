/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { inTab } from './kanban-drawer-dom';
import { HOUR, ago, boardOf, cleanKanban, item, mountKanban, openItem } from './kanban-harness';

cleanKanban();

const waitsOn = (extra: Record<string, unknown> = {}) => ({
  dep: 'D-4', project: 'rotp', item: 'I-12', title: 'Expose the save hook', state: 'open', since: ago(2 * HOUR), reason: null, ...extra,
});

const requestedBy = { dep: 'D-5', project: 'archipelia', item: 'I-7', thread: 'T-2', title: 'Save the game', state: 'open' };

describe('S1 the dependency chip on the cards', { tags: ['standard'] }, () => {
  const cause = (n: number, extra: Record<string, unknown> = {}) => ({ kind: 'dep-waits', id: `D-${n}`, label: `Request number ${n}`, who: 'lead:rotp', ...extra });

  it('shows one chip with a count for the requests that still wait, the titles in the tooltip', async () => {
    await mountKanban({
      items: [
        item('I-1', { waitsOn: waitsOn(), blocked: [cause(4), cause(5), cause(6)] }),
        item('I-2', { waitsOn: waitsOn({ state: 'blocked', reason: 'needs the schema' }), blocked: [{ kind: 'dep-blocked', id: 'D-5', label: 'Save the game', who: 'lead:archipelia' }] }),
      ],
    });
    const board = within(await boardOf());
    const three = board.getByTitle('waiting on rotp: Request number 4; waiting on rotp: Request number 5; waiting on rotp: Request number 6');
    expect(three.textContent).toBe('3');
    expect(board.getByTitle('request D-5 of archipelia is blocked: Save the game').textContent).toBe('1');
  });

  it('never shows a settled, cancelled or plain request, which the board does not export as blocking', async () => {
    await mountKanban({
      items: [
        item('I-3', { waitsOn: waitsOn({ state: 'done' }), blocked: [] }),
        item('I-4', { requestedBy, blocked: [] }),
        item('I-5', { waitsOn: waitsOn({ state: 'cancelled' }), blocked: [] }),
      ],
    });
    await screen.findByText('Item I-3');
    expect(screen.queryByTitle(/waiting on|is blocked/)).toBeNull();
  });
});

describe('S1 the Dependency section of the drawer', { tags: ['standard'] }, () => {
  const blocked = () => item('I-1', { waitsOn: waitsOn({ state: 'blocked', reason: 'needs the schema' }) });

  it('shows the state, the other item as a link to its board, the reason and the age', async () => {
    await mountKanban({ items: [blocked()] });
    const dialog = inTab(await openItem('I-1'), 'Links');
    expect(within(dialog).getByText('Dependency')).toBeTruthy();
    expect(within(dialog).getAllByText('blocked').length).toBeGreaterThan(0);
    expect(within(dialog).getByRole('link', { name: 'rotp/I-12 Expose the save hook' }).getAttribute('href')).toBe('/p/rotp#I-12');
    expect(within(dialog).getByText('Reason: needs the schema')).toBeTruthy();
    expect(within(dialog).getByText('Open for 2h')).toBeTruthy();
  });

  it('has no Dependency section on an item with no dependency', async () => {
    await mountKanban({ items: [item('I-1')] });
    const dialog = inTab(await openItem('I-1'), 'Links');
    expect(within(dialog).queryByText('Dependency')).toBeNull();
  });

  it('cancels as the owner and shows the answer text as a toast', async () => {
    const { posted } = await mountKanban({ items: [blocked()] }, (body) => (body.actor === 'owner' ? { dep: {}, text: 'D-4 is cancelled.' } : undefined));
    const dialog = inTab(await openItem('I-1'), 'Links');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(posted.at(-1)).toMatchObject({ url: '/api/deps/D-4/cancel', body: { actor: 'owner' } }));
    expect(await screen.findByText('D-4 is cancelled.')).toBeTruthy();
  });

  it('offers no Cancel once the request is done', async () => {
    await mountKanban({ items: [item('I-1', { waitsOn: waitsOn({ state: 'done' }) })] });
    const dialog = inTab(await openItem('I-1'), 'Links');
    expect(within(dialog).queryByRole('button', { name: 'Cancel' })).toBeNull();
  });
});
