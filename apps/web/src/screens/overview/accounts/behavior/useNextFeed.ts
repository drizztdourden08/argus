/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { apiRequest } from '../../../../data/api';
import type { AccountsStatus } from '../../../../data/types';
import { useNow } from '../../clock';
import { ACCOUNTS_PATH, NEXT_TICK_MS } from '../next.constants';
import type { NextFeed } from '../next.type';

const statusOf = (answer: unknown): AccountsStatus | null =>
  answer !== null && typeof answer === 'object' && !Array.isArray(answer) ? (answer as AccountsStatus) : null;

const useNextFeed = (): NextFeed => {
  const tick = useNow(NEXT_TICK_MS);
  const [status, setStatus] = useState<AccountsStatus | null>(null);
  const [version, setVersion] = useState(0);
  useEffect(() => {
    let live = true;
    apiRequest<unknown>(ACCOUNTS_PATH).then(
      (answer) => live && setStatus(statusOf(answer)),
      () => live && setStatus(null),
    );
    return () => {
      live = false;
    };
  }, [tick, version]);
  const reload = useCallback(() => setVersion((count) => count + 1), []);
  return { status, reload };
};

export { useNextFeed };
