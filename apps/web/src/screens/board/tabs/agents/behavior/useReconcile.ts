/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { fetchReconcile } from '../../../../../data/agents';
import type { ReconcileFinding } from '../../../../../data/agents';

const useReconcile = (boardKey: string, signature: string): ReconcileFinding[] => {
  const [findings, setFindings] = useState<ReconcileFinding[]>([]);
  useEffect(() => {
    let current = true;
    fetchReconcile(boardKey).then(
      (report) => { if (current) setFindings(report.findings); },
      () => { if (current) setFindings([]); },
    );
    return () => { current = false; };
  }, [boardKey, signature]);
  return findings;
};

export { useReconcile };
