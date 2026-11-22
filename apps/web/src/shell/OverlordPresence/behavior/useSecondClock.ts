/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { CLOCK_MS } from '../OverlordPresence.constants';

const useSecondClock = (): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), CLOCK_MS);
    return () => window.clearInterval(timer);
  }, []);
  return now;
};

export { useSecondClock };
