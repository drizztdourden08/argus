/* @layer renderer-app @kind logic */
import { fieldOf } from '../../../../data/registry';

const textValue = (name: string, text: string): string | string[] => {
  const spec = fieldOf(name);
  if (spec?.content === 'refs') return text.split(/[\s,]+/).filter(Boolean);
  if (spec?.list !== undefined) return text.split('\n').map((line) => line.trim()).filter(Boolean);
  return text.trim();
};

export { textValue };
