/* @layer renderer-app @kind test */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { boardSwitchReducer, runBoardSwitch } from '../../src/screens/board/BoardStage';
import type { SwitchAction, SwitchState } from '../../src/screens/board/BoardStage';

afterEach(() => vi.useRealTimers());

const idle: SwitchState = { shown: 'rotp', target: 'rotp', phase: 'idle' };

const play = (state: SwitchState, actions: SwitchAction[]): SwitchState => actions.reduce(boardSwitchReducer, state);

describe('I-106 the board stage phases', { tags: ['critical'] }, () => {
  it('goes leaving, swap, entering, idle for one switch', () => {
    const leaving = play(idle, [{ type: 'request', key: 'argus' }]);
    expect(leaving).toEqual({ shown: 'rotp', target: 'argus', phase: 'leaving' });
    const swap = play(leaving, [{ type: 'left' }]);
    expect(swap).toEqual({ shown: 'argus', target: 'argus', phase: 'swap' });
    expect(play(swap, [{ type: 'revealed' }]).phase).toBe('entering');
    expect(play(swap, [{ type: 'revealed' }, { type: 'entered' }])).toEqual({ shown: 'argus', target: 'argus', phase: 'idle' });
  });

  it('keeps the latest request and ignores a repeat of the current board', () => {
    expect(play(idle, [{ type: 'request', key: 'rotp' }])).toBe(idle);
    const state = play(idle, [{ type: 'request', key: 'argus' }, { type: 'request', key: 'brock' }, { type: 'left' }]);
    expect(state).toEqual({ shown: 'brock', target: 'brock', phase: 'swap' });
  });

  it('restarts the leave when a request lands during the entry, and ignores late steps', () => {
    const entering = play(idle, [{ type: 'request', key: 'argus' }, { type: 'left' }, { type: 'revealed' }]);
    const next = play(entering, [{ type: 'request', key: 'brock' }]);
    expect(next).toEqual({ shown: 'argus', target: 'brock', phase: 'leaving' });
    expect(play(next, [{ type: 'entered' }, { type: 'revealed' }])).toBe(next);
  });

  it('times each step, waits for the data up to 300 ms and swaps at once under reduced motion', () => {
    vi.useFakeTimers();
    const dispatch = vi.fn();
    runBoardSwitch({ phase: 'leaving', ready: false, reduced: false, dispatch });
    vi.advanceTimersByTime(149);
    expect(dispatch).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(dispatch).toHaveBeenLastCalledWith({ type: 'left' });
    runBoardSwitch({ phase: 'swap', ready: false, reduced: false, dispatch });
    vi.advanceTimersByTime(299);
    expect(dispatch).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(1);
    expect(dispatch).toHaveBeenLastCalledWith({ type: 'revealed' });
    runBoardSwitch({ phase: 'swap', ready: true, reduced: false, dispatch });
    expect(dispatch).toHaveBeenCalledTimes(3);
    runBoardSwitch({ phase: 'entering', ready: true, reduced: true, dispatch });
    expect(dispatch).toHaveBeenLastCalledWith({ type: 'entered' });
  });

  it('cancels a pending step when the phase changes', () => {
    vi.useFakeTimers();
    const dispatch = vi.fn();
    const cancel = runBoardSwitch({ phase: 'leaving', ready: true, reduced: false, dispatch });
    cancel();
    vi.advanceTimersByTime(1000);
    expect(dispatch).not.toHaveBeenCalled();
  });
});
