/* @layer renderer-app @kind logic */
import type { DepDoc } from '../../../../data/deps';
import { FILTER_STATES } from './model.constants';
import type { DepFilter } from './model.type';

const filterDeps = <T extends DepDoc>(docs: readonly T[], filter: DepFilter): T[] => {
  const states = FILTER_STATES[filter];
  return docs.filter((doc) => states === null || states.includes(doc.state));
};

export { filterDeps };
