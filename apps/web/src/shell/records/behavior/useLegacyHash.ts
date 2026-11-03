/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { legacyOpen } from './legacy-open';
import { recordSearch } from './record-search';

const useLegacyHash = (): void => {
  const { hash, search } = useLocation();
  const navigate = useNavigate();
  useEffect(() => {
    const found = legacyOpen(hash);
    if (!found) return;
    void navigate({ search: recordSearch(search, found.code, found.tab), hash: '' }, { replace: true });
  }, [hash, navigate, search]);
};

export { useLegacyHash };
