/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useBoardName } from '../../../data/boards';
import { usePortfolio } from '../../../data/react';

const useProjectName = (): ((key: string) => string) => {
  const { cards } = usePortfolio();
  const boardNameOf = useBoardName();
  return useCallback((key: string) => {
    const named = boardNameOf(key);
    return named === key ? (cards.find((card) => card.key === key)?.name ?? key) : named;
  }, [cards, boardNameOf]);
};

export { useProjectName };
