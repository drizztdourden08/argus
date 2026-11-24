/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Account } from '../../src/screens/overview/accounts/accounts.type';
import { copyDirection, limitLine, mainLimit, nextSteps, whenText } from '../../src/screens/overview/accounts/model';
import { AccountCard } from '../../src/screens/overview/accounts/sub-components/AccountCard';

const account = (extra: Partial<Account>): Account => ({ id: 'a1', key: 'a1', label: 'Alpha', plan: 'Max', ...extra });
const LIMITS = [
  { kind: 'five_hour', percent: 40 },
  { kind: 'weekly_all', percent: 72, resetsAt: null },
];

afterEach(cleanup);

describe('the small account card', { tags: ['critical'] }, () => {
  it('shows the name, the plan and only the highest usage bar', () => {
    render(<AccountCard account={account({ limits: LIMITS })} onOpen={vi.fn()} />);
    expect(screen.getByText('Alpha')).toBeTruthy();
    expect(screen.getByText('Max')).toBeTruthy();
    expect(screen.getAllByRole('progressbar')).toHaveLength(1);
  });

  it('marks the current account and opens its pane on a click', () => {
    const onOpen = vi.fn();
    render(<AccountCard account={account({ active: true, limits: LIMITS })} onOpen={onOpen} />);
    expect(screen.getByText('Current')).toBeTruthy();
    fireEvent.click(screen.getByText('Alpha'));
    expect(onOpen).toHaveBeenCalledWith('a1');
  });

  it('carries its key when dragged', () => {
    render(<AccountCard account={account({})} onOpen={vi.fn()} />);
    const setData = vi.fn();
    fireEvent.dragStart(screen.getByText('Alpha'), { dataTransfer: { setData } });
    expect(setData).toHaveBeenCalledWith('text/plain', 'a1');
  });

  it('shows the limit line only when the limit is confirmed', () => {
    expect(limitLine(account({ limit: { confirmed: false, untilMs: 1 } }))).toBe('');
    expect(limitLine(account({ limit: { confirmed: true, untilMs: Date.now() + 3_600_000 } }))).toMatch(/^At limit until /);
  });
});

describe('the model of the switch tab', { tags: ['critical'] }, () => {
  it('picks the highest limit as the main bar', () => {
    expect(mainLimit(account({ limits: LIMITS }))?.kind).toBe('weekly_all');
    expect(mainLimit(account({}))).toBeNull();
  });

  it('maps the run phase to the stepper step', () => {
    expect(nextSteps('handover-asked').currentId).toBe('handover-asked');
    expect(nextSteps('limit-confirmed').steps[1]?.label).toBe('Limit confirmed');
    expect(nextSteps('switched').complete).toBe(true);
    expect(nextSteps().currentId).toBeUndefined();
  });

  it('words the when and the copy direction', () => {
    expect(whenText({ kind: 'percent', percent: 98 })).toBe('At 98% used');
    expect(whenText({ kind: 'unknown' })).toBe('When is not known yet');
    expect(copyDirection('rtl', 'left', 'right')).toEqual({ from: 'right', to: 'left' });
    expect(copyDirection('both', 'left', 'right')).toBeNull();
  });
});
