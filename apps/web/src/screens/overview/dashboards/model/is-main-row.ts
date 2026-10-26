/* @layer renderer-app @kind logic */
import type { RegistryItem } from './model.type';

const isMainRow = (row: RegistryItem): boolean => row.project === 'portfolio' && row.role === 'coordinator';

export { isMainRow };
