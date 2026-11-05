/* @layer renderer-app @kind logic */
import type { FilterClause } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { ClauseField, ClauseFilters } from '../BoardTabsRow.type';
import { descriptorOf } from './descriptor-of';
import { fromClauses } from './from-clauses';
import { settled } from './settled';
import { toClauses } from './to-clauses';

const useClauseFilters = <F>(fields: readonly ClauseField<F>[], filters: F, patch: (values: Partial<F>) => void): ClauseFilters => {
  const stored = toClauses(fields, filters);
  const outside = JSON.stringify(fromClauses(fields, stored));
  const [local, setLocal] = useState<readonly FilterClause[]>(stored);
  const [seen, setSeen] = useState(outside);
  if (seen !== outside) {
    setSeen(outside);
    if (JSON.stringify(fromClauses(fields, local)) !== outside) setLocal(stored);
  }
  const onChange = (next: readonly FilterClause[]) => {
    const clauses = settled(fields, next);
    setLocal(clauses);
    patch(fromClauses(fields, clauses));
  };
  return { schema: fields.map(descriptorOf), clauses: local, onChange, count: stored.length };
};

export { useClauseFilters };
