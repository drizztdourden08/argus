/* @layer renderer-app @kind logic */
import type { TagCategoryColor } from '@drizztdourden08/tessera';
import { CATEGORY_COLORS } from './look.constants';

const categoryColor = (name: string | undefined): TagCategoryColor | undefined => CATEGORY_COLORS.find((color) => color === name);

export { categoryColor };
