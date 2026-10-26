/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { usePortfolio } from '../../../data/react';

const useProjectName = (): ((key: string) => string) => {
  const { cards } = usePortfolio();
  return useCallback((key: string) => cards.find((card) => card.key === key)?.name ?? key, [cards]);
};

export { useProjectName };
