/* @layer renderer-app @kind test */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { runBoardReveal } from '../../src/screens/board/BoardStage';

afterEach(() => vi.useRealTimers());

describe('I-106 the board stage reveal', { tags: ['critical'] }, () => {
  it('reveals at once when the data is ready or motion is reduced', () => {
    const reveal = vi.fn();
    runBoardReveal({ ready: true, reduced: false, reveal });
    runBoardReveal({ ready: false, reduced: true, reveal });
    expect(reveal).toHaveBeenCalledTimes(2);
  });

  it('waits for the data up to 300 ms', () => {
    vi.useFakeTimers();
    const reveal = vi.fn();
    runBoardReveal({ ready: false, reduced: false, reveal });
    vi.advanceTimersByTime(299);
    expect(reveal).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(reveal).toHaveBeenCalledTimes(1);
  });

  it('cancels a pending reveal', () => {
    vi.useFakeTimers();
    const reveal = vi.fn();
    runBoardReveal({ ready: false, reduced: false, reveal })();
    vi.advanceTimersByTime(1000);
    expect(reveal).not.toHaveBeenCalled();
  });
});
