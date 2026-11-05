/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { listRuns } from '../../../../data/runs';
import type { RunRow } from '../../../../data/runs';
import { usePortfolio } from '../../../../data/react';
import { RUNS_REFRESH_MS } from '../boards-run.constants';

const useRunningRuns = (): RunRow[] => {
  const { cards } = usePortfolio();
  const [runs, setRuns] = useState<RunRow[]>([]);
  useEffect(() => {
    let live = true;
    const timer = setTimeout(() => {
      listRuns(null, { status: 'running' }).then(
        (rows) => live && setRuns(rows),
        () => undefined,
      );
    }, RUNS_REFRESH_MS);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [cards]);
  return runs;
};

export { useRunningRuns };
