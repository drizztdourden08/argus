/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { boardUrl } from '../../behavior/board-url';

const useSwitchBoard = (): ((key: string) => void) => {
  const navigate = useNavigate();
  const { hash } = useLocation();
  return useCallback((key: string) => void navigate(boardUrl(key, hash)), [navigate, hash]);
};

export { useSwitchBoard };
