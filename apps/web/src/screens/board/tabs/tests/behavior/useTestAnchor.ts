/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { ANCHOR_PATTERN, TAB_HASH } from '../tests.constants';
import type { TestAnchor } from '../tests.type';

const useTestAnchor = (): TestAnchor => {
  const { hash, search } = useLocation();
  const navigate = useNavigate();
  const wanted = hash.slice(1);
  const open = useCallback((testId: string) => void navigate({ search, hash: testId }, { replace: true }), [navigate, search]);
  const close = useCallback(() => void navigate({ search, hash: TAB_HASH }, { replace: true }), [navigate, search]);
  return { selected: ANCHOR_PATTERN.test(wanted) ? wanted : null, open, close };
};

export { useTestAnchor };
