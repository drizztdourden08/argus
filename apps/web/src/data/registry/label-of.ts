/* @layer renderer-app @kind logic */
import { fieldOf } from './field-of';

const labelOf = (field: string, value: string): string => {
  const spec = fieldOf(field);
  return spec?.labels?.[spec.values?.indexOf(value) ?? -1] ?? value;
};

export { labelOf };
