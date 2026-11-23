/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { apiRequest } from '../../../../data/api';
import { COMPARE_PATH } from '../compare.constants';
import type { CompareDoc } from '../compare.type';

const useCompareFeed = (refresh: string): CompareDoc | null => {
  const [compare, setCompare] = useState<CompareDoc | null>(null);
  useEffect(() => {
    let live = true;
    apiRequest<CompareDoc>(COMPARE_PATH).then(
      (doc) => live && setCompare(doc),
      () => live && setCompare(null),
    );
    return () => {
      live = false;
    };
  }, [refresh]);
  return compare;
};

export { useCompareFeed };
