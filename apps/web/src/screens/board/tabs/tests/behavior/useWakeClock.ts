/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { WAKE_TICK_MS } from '../tests.constants';

const useWakeClock = (active: boolean): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!active) return undefined;
    setNow(Date.now());
    const timer = setInterval(() => setNow(Date.now()), WAKE_TICK_MS);
    return () => clearInterval(timer);
  }, [active]);
  return now;
};

export { useWakeClock };
