/* @layer renderer-app @kind logic */
import type { DepDoc } from '../../../../../data/deps';
import type { WithId } from '../../../../../data/react';
import type { DepSides } from '../board.type';

const depsOfBoard = (deps: readonly WithId<DepDoc>[], key: string): DepSides => ({
  incoming: deps.filter((dep) => dep.to.project === key),
  outgoing: deps.filter((dep) => dep.from.project === key),
});

export { depsOfBoard };
