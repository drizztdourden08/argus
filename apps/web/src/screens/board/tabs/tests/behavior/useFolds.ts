/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { Folds } from '../tests.type';

const useFolds = (): Folds => {
  const [overrides, setOverrides] = useState<Record<string, boolean>>({});
  const isOpen = useCallback((key: string, fallback: boolean) => overrides[key] ?? fallback, [overrides]);
  const set = useCallback((key: string, open: boolean) => setOverrides((current) => ({ ...current, [key]: open })), []);
  return { isOpen, set };
};

export { useFolds };
