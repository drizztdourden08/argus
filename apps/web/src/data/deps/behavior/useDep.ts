/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE, useDoc } from '../../react';
import type { DocLookup } from '../../react';
import { DEPS_COLLECTION } from '../deps.constants';
import type { DepDoc } from '../deps.type';

const useDep = (id: string): DocLookup<DepDoc> => useDoc<DepDoc>(PORTFOLIO_SCOPE, DEPS_COLLECTION, id);

export { useDep };
