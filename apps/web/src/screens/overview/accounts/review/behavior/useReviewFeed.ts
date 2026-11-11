/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { apiRequest, errorMessage } from '../../../../../data/api';
import type { ReviewDoc } from '../../../../../data/types';
import { REVIEW_PATH } from '../review.constants';
import type { FeedState, ReviewFeed } from '../review.type';

const useReviewFeed = (): ReviewFeed => {
  const [state, setState] = useState<FeedState>({ data: null, loading: true, error: null });

  const rescan = useCallback(() => {
    setState((prev) => ({ ...prev, loading: true }));
    apiRequest<ReviewDoc>(REVIEW_PATH).then(
      (data) => setState({ data, loading: false, error: null }),
      (failure: unknown) => setState((prev) => ({ ...prev, loading: false, error: errorMessage(failure) })),
    );
  }, []);

  useEffect(rescan, [rescan]);

  return { ...state, rescan };
};

export { useReviewFeed };
