/* @layer renderer-app @kind logic */
import type { Said } from './model.type';

const failed = (message: string): Said => ({ variant: 'danger', message });

export { failed };
