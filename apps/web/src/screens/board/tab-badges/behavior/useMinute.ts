/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { CLOCK_MS } from '../tab-badges.constants';

const useMinute = (): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), CLOCK_MS);
    return () => clearInterval(timer);
  }, []);
  return now;
};

export { useMinute };
