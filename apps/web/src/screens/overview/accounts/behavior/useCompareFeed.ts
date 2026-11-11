/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../data/api';
import type { CompareDoc } from '../../../../data/types';
import { COMPARE_PATH } from '../compare.constants';
import type { CompareFeed } from '../compare.type';
import type { FeedState } from './compare-feed.type';

const useCompareFeed = (): CompareFeed => {
  const [state, setState] = useState<FeedState>({ data: null, loading: true, error: null });

  const rescan = useCallback(() => {
    setState((prev) => ({ ...prev, loading: true }));
    apiRequest<CompareDoc>(COMPARE_PATH).then(
      (data) => setState({ data, loading: false, error: null }),
      (failure: unknown) => setState((prev) => ({ ...prev, loading: false, error: errorMessage(failure) })),
    );
  }, []);

  useEffect(rescan, [rescan]);

  return { ...state, rescan };
};

export { useCompareFeed };
