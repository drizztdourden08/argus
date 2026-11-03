/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { parseRecord } from '../../../shell';

const useEntryAnchor = (open: (id: string) => void): void => {
  const { hash, search } = useLocation();
  const navigate = useNavigate();
  useEffect(() => {
    const ref = parseRecord(hash.slice(1));
    if (ref?.kind !== 'entry' || ref.board !== null) return;
    open(ref.id);
    void navigate({ search, hash: '' }, { replace: true });
  }, [hash, navigate, open, search]);
};

export { useEntryAnchor };
