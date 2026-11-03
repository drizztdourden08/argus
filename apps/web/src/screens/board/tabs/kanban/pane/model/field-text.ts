/* @layer renderer-app @kind logic */
import { fieldOf } from '../../../../../../data/registry';

const fieldText = (name: string, value: unknown): string => {
  if (Array.isArray(value)) return value.join(fieldOf(name)?.content === 'refs' ? ', ' : '\n');
  return typeof value === 'string' ? value : '';
};

export { fieldText };
