/* @layer renderer-app @kind logic */
import { MODEL_FAMILIES } from './model.constants';
import type { ModelFamily } from './model.type';

const modelFamily = (model: string): ModelFamily => MODEL_FAMILIES.find((family) => model.toLowerCase().includes(family)) ?? 'other';

export { modelFamily };
