/* @layer renderer-app @kind logic */
import type { FilterClause } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { TabField } from '../BoardTabsRow.type';
import { clauseOfField } from './clause-of-field';
import { clausePatch } from './clause-patch';
import { fieldIsSet } from './field-is-set';

const useFieldClauses = <F>(fields: readonly TabField<F>[], filters: F, patch: (values: Partial<F>) => void) => {
  const [added, setAdded] = useState<readonly string[]>([]);
  const stored = fields.filter((field) => fieldIsSet(field, filters) && !added.includes(field.path)).map((field) => field.path);
  const clauses: FilterClause[] = [...added, ...stored].flatMap((path) =>
    fields.filter((field) => field.path === path).map((field) => clauseOfField(field, filters)),
  );
  const onChange = (next: readonly FilterClause[]) => {
    setAdded(next.map((clause) => clause.path));
    patch(clausePatch(fields, next, filters));
  };
  return { clauses, onChange };
};

export { useFieldClauses };
