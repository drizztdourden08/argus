/* @layer renderer-app @kind logic */
import type { Said } from './model.type';

const succeeded = (message: string): Said => ({ variant: 'success', message });

export { succeeded };
