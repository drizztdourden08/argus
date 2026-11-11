/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { DecisionState, Decisions } from '../review.type';

const useDecisions = (): DecisionState => {
  const [decisions, setDecisions] = useState<Decisions>({});

  const choose = useCallback((rowId: string, choice: string, fallback: string) => {
    setDecisions((prev) => {
      const { [rowId]: _gone, ...rest } = prev;
      return choice === fallback ? rest : { ...rest, [rowId]: choice };
    });
  }, []);

  return { decisions, choose };
};

export { useDecisions };
