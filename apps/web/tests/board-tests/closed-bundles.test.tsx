/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cardOf, cleanAfterEach, detailsOf } from './board-dom';
import { buildDoc, doneDoc, testDoc } from './fixtures';
import { mountTab } from './mount-tab';

cleanAfterEach();

const closedBoard = () => ({
  builds: [buildDoc('open-1'), buildDoc('old-1', { closedAt: '2026-10-25T10:00:00.000Z', closedBy: 'lead', closeNote: 'landed' })],
  tests: [
    testDoc('T-1', { build: 'open-1' }),
    doneDoc('T-2', 'pass', 'ok', { build: 'open-1' }),
    testDoc('T-3', { build: 'old-1' }),
    testDoc('T-4', { build: 'old-1' }),
    doneDoc('T-5', 'fail', 'no', { build: 'old-1' }),
  ],
});

describe('closed bundles', () => {
  it('are out of the Tests tab and the summary by default', async () => {
    await mountTab(closedBoard());
    expect(await screen.findByText('Test T-1')).toBeTruthy();
    expect(screen.queryByText('Test T-3')).toBeNull();
    expect(screen.queryByText('old-1')).toBeNull();
    expect(screen.getByText('1 to run / 2')).toBeTruthy();
  });

  it('have no toggle when no bundle is closed', async () => {
    await mountTab({ builds: [buildDoc('open-1')], tests: [testDoc('T-1', { build: 'open-1' })] });
    expect(await screen.findByText('Test T-1')).toBeTruthy();
    expect(screen.queryByRole('switch', { name: /Show closed/ })).toBeNull();
  });

  it('show as a muted card with Show closed, still not counted', async () => {
    const { container } = await mountTab(closedBoard());
    fireEvent.click(await screen.findByRole('switch', { name: 'Show closed (1)' }));
    const card = cardOf(container, 'old-1');
    expect(within(card).getByText('Closed')).toBeTruthy();
    expect(card.textContent).toContain('by lead');
    expect(card.textContent).toContain('2 to run / 1 fail');
    expect(screen.getByText('1 to run / 2')).toBeTruthy();
  });

  it('unfold to read-only rows: no outcome buttons, no Play, no Hand', async () => {
    const { container } = await mountTab(closedBoard());
    fireEvent.click(await screen.findByRole('switch', { name: 'Show closed (1)' }));
    const card = cardOf(container, 'old-1');
    expect(detailsOf(card).open).toBe(false);
    fireEvent.click(card.querySelector('summary') as Element);
    expect(detailsOf(card).open).toBe(true);
    expect(within(card).getByText('Test T-3')).toBeTruthy();
    expect(within(card).queryByRole('button', { name: /Pass|Fail|Partial|Blocked|Play|Hand|again/ })).toBeNull();
  });

  it('hide again when the toggle is turned off', async () => {
    await mountTab(closedBoard());
    const toggle = await screen.findByRole('switch', { name: 'Show closed (1)' });
    fireEvent.click(toggle);
    expect(screen.getByText('old-1')).toBeTruthy();
    fireEvent.click(toggle);
    expect(screen.queryByText('old-1')).toBeNull();
  });
});
