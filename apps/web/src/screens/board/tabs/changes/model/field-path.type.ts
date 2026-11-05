/* @layer renderer-app @kind types */
import type { ChangesFilters } from '../changes.type';

type FieldPath = keyof Omit<ChangesFilters, 'query'>;

export type { FieldPath };
