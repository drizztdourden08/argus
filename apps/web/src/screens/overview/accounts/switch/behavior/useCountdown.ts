/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';

const useCountdown = (endsAt: number | null): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (endsAt === null) return undefined;
    setNow(Date.now());
    const timer = window.setInterval(() => setNow(Date.now()), 500);
    return () => window.clearInterval(timer);
  }, [endsAt]);
  return endsAt === null ? 0 : endsAt - now;
};

export { useCountdown };
