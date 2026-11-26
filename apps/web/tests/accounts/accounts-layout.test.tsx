/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Account } from '../../src/screens/overview/accounts/accounts.type';
import { clearSlot, copyDirection, limitLine, mainLimit, nextPair, nextSteps, placeSlot, slotHint, whenText } from '../../src/screens/overview/accounts/model';
import { AccountCard } from '../../src/screens/overview/accounts/sub-components/AccountCard';
import { AccountSlot } from '../../src/screens/overview/accounts/sub-components/AccountSlot';

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
    expect(setData).toHaveBeenCalledWith('application/x-argus-account', 'a1');
  });

  it('shows the limit line only when the limit is confirmed', () => {
    expect(limitLine(account({ limit: { confirmed: false, untilMs: 1 } }))).toBe('');
    expect(limitLine(account({ limit: { confirmed: true, untilMs: Date.now() + 3_600_000 } }))).toMatch(/^At limit until /);
  });
});

describe('the account slot', { tags: ['critical'] }, () => {
  const beta = account({ id: 'b2', key: 'b2', label: 'Beta' });

  it('shows the held account as the small card and clears on its button', () => {
    const onClear = vi.fn();
    render(<AccountSlot label="From" accounts={[account({ limits: LIMITS }), beta]} value="a1" hint="Your pick" onPick={vi.fn()} onClear={onClear} />);
    expect(screen.getByText('Alpha')).toBeTruthy();
    expect(screen.getByText('Max')).toBeTruthy();
    expect(screen.getAllByRole('progressbar')).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: 'Clear From' }));
    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it('has no clear button while empty and takes a dropped account', () => {
    const onPick = vi.fn();
    render(<AccountSlot label="To" accounts={[account({}), beta]} value={null} hint="Auto: Beta" onPick={onPick} onClear={vi.fn()} />);
    expect(screen.queryByRole('button', { name: 'Clear To' })).toBeNull();
    expect(screen.getByText('Auto: Beta')).toBeTruthy();
    const dataTransfer = { types: ['application/x-argus-account'], getData: () => 'b2' };
    fireEvent.drop(screen.getByRole('group', { name: 'To' }), { dataTransfer });
    expect(onPick).toHaveBeenCalledWith('b2');
  });

  it('carries the key of the held account when it is dragged out', () => {
    render(<AccountSlot label="From" accounts={[account({}), beta]} value="a1" onPick={vi.fn()} onClear={vi.fn()} />);
    const setData = vi.fn();
    fireEvent.dragStart(screen.getByText('Alpha'), { dataTransfer: { setData } });
    expect(setData).toHaveBeenCalledWith('application/x-argus-account', 'a1');
  });
});

describe('the pair of slots', { tags: ['critical'] }, () => {
  it('places an account, swaps when it comes from the other slot and clears one side', () => {
    expect(placeSlot({ left: null, right: null }, 'right', 'b')).toEqual({ left: null, right: 'b' });
    expect(placeSlot({ left: 'a', right: 'b' }, 'right', 'a')).toEqual({ left: 'b', right: 'a' });
    expect(placeSlot({ left: 'a', right: 'b' }, 'left', 'b')).toEqual({ left: 'b', right: 'a' });
    expect(placeSlot({ left: 'a', right: null }, 'right', 'a')).toEqual({ left: null, right: 'a' });
    expect(clearSlot({ left: 'a', right: 'b' }, 'left')).toEqual({ left: null, right: 'b' });
  });

  it('shows only the ends the owner picked and keeps the auto pick as the hint', () => {
    const auto = nextPair({ next: { from: 'a', to: 'b', source: 'auto', plan: null } });
    expect(auto.picked).toEqual({ left: null, right: null });
    expect(auto.auto).toEqual({ left: 'a', right: 'b' });
    const half = nextPair({ next: { from: 'a', to: 'b', source: 'owner', plan: { from: null, to: 'b' } } });
    expect(half.picked).toEqual({ left: null, right: 'b' });
    const whole = nextPair({ next: { from: 'a', to: 'b', source: 'owner' } });
    expect(whole.picked).toEqual({ left: 'a', right: 'b' });
    expect(nextPair(null).picked).toEqual({ left: null, right: null });
  });

  it('words the line under a slot', () => {
    const accounts = [account({})];
    expect(slotHint(accounts, 'a1', null)).toBe('Your pick');
    expect(slotHint(accounts, null, 'a1')).toBe('Auto: Alpha');
    expect(slotHint(accounts, null, null)).toBe('Auto');
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
    expect(nextSteps().currentId).toBeNull();
  });

  it('words the when and the copy direction', () => {
    expect(whenText({ kind: 'percent', percent: 98 })).toBe('At 98% used');
    expect(whenText({ kind: 'unknown' })).toBe('When is not known yet');
    expect(copyDirection('rtl', 'left', 'right')).toEqual({ from: 'right', to: 'left' });
    expect(copyDirection('both', 'left', 'right')).toBeNull();
  });
});
