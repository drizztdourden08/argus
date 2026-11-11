/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';

const scopeMap = (rows: readonly ReviewRow[]): Map<string, ReviewRow[]> => {
  const map = new Map<string, ReviewRow[]>();
  const above: ReviewRow[] = [];
  for (const row of rows) {
    while (above.length > 0 && (above.at(-1)?.depth ?? 0) >= row.depth) above.pop();
    for (const parent of above) map.get(parent.id)?.push(row);
    map.set(row.id, []);
    above.push(row);
  }
  return map;
};

export { scopeMap };
