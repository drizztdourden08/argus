/* @layer renderer-app @kind logic */
import type { ProcRule, ProcsMeta } from '../../../../data/types';

const rulesOf = (meta: ProcsMeta | null): ProcRule[] =>
  Array.isArray(meta?.rules) ? meta.rules.filter((rule) => rule.name) : [];

export { rulesOf };
