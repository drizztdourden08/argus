/* @layer renderer-app @kind logic */
import type { WantsMap } from './wants-map.type';

const add = (map: WantsMap, scope: string, coll: string): void => {
  map.set(scope, (map.get(scope) ?? new Set<string>()).add(coll));
};

const remove = (map: WantsMap, scope: string, coll: string): void => {
  const colls = map.get(scope);
  if (colls?.delete(coll) && colls.size === 0) map.delete(scope);
};

const wantsMap = { add, remove };

export { wantsMap };
