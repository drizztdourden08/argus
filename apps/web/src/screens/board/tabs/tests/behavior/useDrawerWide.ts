/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { readStored, writeStored } from '../../../shared';
import { STORE_KEYS } from '../model';
import type { DrawerWide } from '../tests.type';

const useDrawerWide = (): DrawerWide => {
  const [wide, setWide] = useState(() => readStored(STORE_KEYS.wide) === '1');
  const toggle = useCallback(() => {
    writeStored(STORE_KEYS.wide, wide ? '0' : '1');
    setWide(!wide);
  }, [wide]);
  return { wide, toggle };
};

export { useDrawerWide };
