/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { TICK_MS } from './clock.constants';

const useNow = (intervalMs: number = TICK_MS): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), intervalMs);
    return () => clearInterval(timer);
  }, [intervalMs]);
  return now;
};

export { useNow };
