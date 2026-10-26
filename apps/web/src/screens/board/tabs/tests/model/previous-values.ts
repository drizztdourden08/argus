/* @layer renderer-app @kind logic */
import type { DocData } from '../../../../../data/api';
import type { EditChange, TestDoc } from './model.type';

const previousValues = (test: TestDoc, fields: readonly (keyof EditChange)[]): DocData =>
  Object.fromEntries(fields.map((field) => [field, test[field] ?? (field === 'build' ? null : '')]));

export { previousValues };
