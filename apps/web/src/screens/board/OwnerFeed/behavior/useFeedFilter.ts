/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { FILTER_STORAGE_KEY } from '../OwnerFeed.constants';
import type { FeedFilter } from '../OwnerFeed.type';

const isFilter = (value: string | null): value is FeedFilter => value === 'all' || value === 'you' || value === 'log';

const readFilter = (): FeedFilter => {
  try {
    const stored = window.localStorage.getItem(FILTER_STORAGE_KEY);
    return isFilter(stored) ? stored : 'all';
  } catch {
    return 'all';
  }
};

const saveFilter = (filter: FeedFilter): boolean => {
  try {
    window.localStorage.setItem(FILTER_STORAGE_KEY, filter);
    return true;
  } catch {
    return false;
  }
};

const useFeedFilter = (): [FeedFilter, (filter: FeedFilter) => void] => {
  const [filter, setFilter] = useState<FeedFilter>(readFilter);
  const choose = useCallback((next: FeedFilter) => {
    setFilter(next);
    saveFilter(next);
  }, []);
  return [filter, choose];
};

export { useFeedFilter };
