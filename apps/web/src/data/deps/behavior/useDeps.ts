/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE, useCollection } from '../../react';
import type { CollectionResult } from '../../react';
import { DEPS_COLLECTION } from '../deps.constants';
import type { DepDoc } from '../deps.type';

const useDeps = (): CollectionResult<DepDoc> => useCollection<DepDoc>(PORTFOLIO_SCOPE, DEPS_COLLECTION);

export { useDeps };
