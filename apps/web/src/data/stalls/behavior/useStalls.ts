/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE, useCollection } from '../../react';
import type { CollectionResult } from '../../react';
import { STALLS_COLLECTION } from '../stalls.constants';
import type { StallDoc } from '../stalls.type';

const useStalls = (): CollectionResult<StallDoc> => useCollection<StallDoc>(PORTFOLIO_SCOPE, STALLS_COLLECTION);

export { useStalls };
