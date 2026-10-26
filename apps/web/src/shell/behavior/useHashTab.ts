/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router';
import type { HashTab } from '../shell.type';

const useHashTab = (tabs: readonly string[], fallback: string): HashTab => {
  const { hash, search } = useLocation();
  const navigate = useNavigate();
  const wanted = hash.slice(1);
  const select = useCallback((next: string) => void navigate({ search, hash: next }), [navigate, search]);
  return { tab: tabs.includes(wanted) ? wanted : fallback, select };
};

export { useHashTab };
