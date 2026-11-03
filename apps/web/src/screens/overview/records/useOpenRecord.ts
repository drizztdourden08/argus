/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useNavigate } from 'react-router';
import { openHref } from './open-href';
import type { RowTarget } from './records.type';

const useOpenRecord = (): ((target: RowTarget) => void) => {
  const navigate = useNavigate();
  return useCallback((target: RowTarget) => {
    const href = openHref(target.project, target.open, target.tab);
    if (href) void navigate(href);
  }, [navigate]);
};

export { useOpenRecord };
