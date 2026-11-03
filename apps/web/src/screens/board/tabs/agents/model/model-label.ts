/* @layer renderer-app @kind logic */
import { modelFamily } from './model-family';

const modelLabel = (model: string): string => {
  const family = modelFamily(model);
  return family === 'other' ? model : family;
};

export { modelLabel };
