/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { NOW_TICK_MS } from '../shared.constants';

const useNow = (running: boolean): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!running) return undefined;
    const timer = setInterval(() => setNow(Date.now()), NOW_TICK_MS);
    return () => clearInterval(timer);
  }, [running]);
  return now;
};

export { useNow };
