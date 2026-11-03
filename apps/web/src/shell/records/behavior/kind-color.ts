/* @layer renderer-app @kind logic */
import type { TagCategoryColor } from '@drizztdourden08/tessera';
import type { RegistryKind } from '../../../data/registry';
import { categoryColor } from './category-color';
import { TONE_HUES } from './kind-color.constants';

const kindColor = (kind: RegistryKind): TagCategoryColor | undefined => categoryColor(kind.look.color) ?? (kind.look.tone ? TONE_HUES[kind.look.tone] : undefined);

export { kindColor };
