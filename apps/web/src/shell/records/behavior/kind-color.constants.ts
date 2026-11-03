/* @layer renderer-app @kind constants */
import type { TagCategoryColor } from '@drizztdourden08/tessera';

const TONE_HUES: Readonly<Record<string, TagCategoryColor>> = {
  danger: 'rose',
  warning: 'amber',
  info: 'blue',
  success: 'green',
};

export { TONE_HUES };
