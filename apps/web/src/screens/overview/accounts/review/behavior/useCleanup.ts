/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../../data/api';
import type { CleanupDoc } from '../../../../../data/types';
import { candidateKey } from '../model';
import { CLEANUP_PATH, DEFAULT_DAYS } from '../review.constants';
import type { CleanupState } from '../review.type';

const useCleanup = (): CleanupState => {
  const [days, setDays] = useState(DEFAULT_DAYS);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [found, setFound] = useState<CleanupDoc | null>(null);
  const [picked, setPicked] = useState<ReadonlySet<string>>(new Set());

  const find = useCallback(() => {
    setLoading(true);
    setError(null);
    apiRequest<CleanupDoc>(`${CLEANUP_PATH}?days=${Math.max(1, Math.round(days))}`)
      .then(
        (doc) => {
          setFound(doc);
          setPicked(new Set(doc.candidates.map(candidateKey)));
        },
        (failure: unknown) => setError(errorMessage(failure)),
      )
      .finally(() => setLoading(false));
  }, [days]);

  const toggle = useCallback((key: string) => {
    setPicked((prev) => {
      const next = new Set(prev);
      if (!next.delete(key)) next.add(key);
      return next;
    });
  }, []);

  return { days, loading, error, found, picked, setDays, find, toggle };
};

export { useCleanup };
