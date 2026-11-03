/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useRef, useState } from 'react';
import { errorMessage } from '../../api';
import type { RemoteRead } from '../remote.type';

const useRemoteRead = <T,>(load: () => Promise<T>, every = 0): RemoteRead<T> => {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState('');
  const live = useRef(true);

  const reload = useCallback(() => {
    load().then(
      (read) => {
        if (!live.current) return;
        setData(read);
        setError('');
      },
      (failure: unknown) => {
        if (live.current) setError(errorMessage(failure));
      },
    );
  }, [load]);

  useEffect(() => {
    live.current = true;
    reload();
    const timer = every > 0 ? window.setInterval(reload, every) : 0;
    return () => {
      live.current = false;
      window.clearInterval(timer);
    };
  }, [reload, every]);

  return { data, error, reload };
};

export { useRemoteRead };
