/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { usePortfolio } from '../../../data/react';
import { boardUrl } from '../../behavior/board-url';
import type { ShellMode, ShellModeView } from '../AppShell.type';
import { boardsTarget } from './boards-target';
import { parseShellMode } from './parse-shell-mode';
import { useLastBoard } from './useLastBoard';

const useShellMode = (): ShellModeView => {
  const { pathname } = useLocation();
  const navigate = useNavigate();
  const { cards } = usePortfolio();
  const current = parseShellMode(pathname);
  const target = boardsTarget(useLastBoard(current.boardKey), cards);
  const select = useCallback(
    (mode: ShellMode) => {
      if (mode === 'overview') void navigate('/');
      else if (target) void navigate(boardUrl(target));
    },
    [navigate, target],
  );
  return { ...current, boardsAvailable: target !== null, select };
};

export { useShellMode };
