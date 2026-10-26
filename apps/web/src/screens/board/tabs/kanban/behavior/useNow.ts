/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { NOW_TICK_MS } from '../kanban.constants';

const useNow = (): number => {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), NOW_TICK_MS);
    return () => clearInterval(timer);
  }, []);
  return now;
};

export { useNow };
