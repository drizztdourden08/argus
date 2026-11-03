/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { OwnerAsk, OwnerAskState } from '../owner-links.type';

const useOwnerAsk = (): OwnerAskState => {
  const [ask, setAsk] = useState<OwnerAsk | null>(null);
  const open = useCallback((next: OwnerAsk) => setAsk(next), []);
  const close = useCallback(() => setAsk(null), []);
  return { ask, open, close };
};

export { useOwnerAsk };
