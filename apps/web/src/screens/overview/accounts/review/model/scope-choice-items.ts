/* @layer renderer-app @kind logic */
import type { MenuItem } from '@drizztdourden08/tessera';
import type { ScopeMenuArgs } from '../review-tree-props.type';
import { chooseSide } from './choose-side';
import { scopeOf } from './scope-of';
import { SIDE_ITEMS } from './scope-choice-items.constants';
import { sideState } from './side-state';

const scopeChoiceItems = (args: ScopeMenuArgs): MenuItem[] => {
  const { row, env } = args;
  const scope = scopeOf(row, env.scopes);
  return SIDE_ITEMS.filter((item) => sideState(scope, env.decisions, item.side, env.pair).enabled).map(
    (item): MenuItem => ({ id: item.id, label: item.label, icon: item.icon, onSelect: () => chooseSide({ rows: scope, side: item.side, pair: env.pair, revert: false, onChoose: env.onChoose }) }),
  );
};

export { scopeChoiceItems };
