/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection, useDoc } from '../../../../data/react';
import type { ProcDoc, ProcsMeta } from '../../../../data/types';
import { sortProcs } from '../model';
import type { ProcessFeed } from '../processes.type';

const useProcessFeed = (): ProcessFeed => {
  const procs = useCollection<ProcDoc>(PORTFOLIO_SCOPE, 'procs');
  const meta = useDoc<ProcsMeta>(PORTFOLIO_SCOPE, 'meta', 'procs');
  const sorted = useMemo(() => sortProcs(procs.docs), [procs.docs]);
  return { ready: procs.ready, procs: sorted, meta: meta.doc };
};

export { useProcessFeed };
