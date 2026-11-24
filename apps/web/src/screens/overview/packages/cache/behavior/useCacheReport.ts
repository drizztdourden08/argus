/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useRef, useState } from 'react';
import { readCache } from '../../../../../data/cache';
import type { CacheReport } from '../../../../../data/cache';
import type { CacheReportState } from '../cache.type';
import { cacheError } from '../model';

const useCacheReport = (): CacheReportState => {
  const [report, setReport] = useState<CacheReport | null>(null);
  const [error, setError] = useState('');
  const live = useRef(true);

  const reload = useCallback(() => {
    readCache().then(
      (read) => {
        if (!live.current) return;
        setReport(read);
        setError('');
      },
      (failure: unknown) => {
        if (live.current) setError(cacheError(failure, (detail) => `The build cache could not be read: ${detail}.`));
      },
    );
  }, []);

  useEffect(() => {
    live.current = true;
    reload();
    return () => {
      live.current = false;
    };
  }, [reload]);

  return { report, error, reload };
};

export { useCacheReport };
