/* @layer renderer-app @kind logic */
import type { TagCategoryColor } from '@drizztdourden08/tessera';
import { ACCENTS } from './model.constants';

const configAccent = (accent: string | null | undefined): TagCategoryColor | undefined => ACCENTS.find((entry) => entry === accent);

export { configAccent };
