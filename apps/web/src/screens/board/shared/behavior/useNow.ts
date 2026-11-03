/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { NOW_TICK_MS } from '../shared.constants';

const useNow = (running: boolean, tickMs: number = NOW_TICK_MS): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!running) return undefined;
    const timer = setInterval(() => setNow(Date.now()), tickMs);
    return () => clearInterval(timer);
  }, [running, tickMs]);
  return now;
};

export { useNow };
