/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { useSwitchLink } from './useSwitchLink';

interface SwitchTarget {
  key: string | null;
  open: (key: string) => void;
  close: () => void;
}

const useSwitchTarget = (): SwitchTarget => {
  const link = useSwitchLink();
  const [key, setKey] = useState<string | null>(null);
  const { target, clear } = link;
  useEffect(() => {
    if (!target) return;
    setKey(target);
    clear();
  }, [target, clear]);
  return { key, open: setKey, close: () => setKey(null) };
};

export { useSwitchTarget };
export type { SwitchTarget };
