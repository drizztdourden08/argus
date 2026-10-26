/* @layer renderer-app @kind logic */
import type { Entry } from '../../../data/types';

const hasContext = (entry: Entry): boolean => typeof entry.context === 'string' && entry.context.trim() !== '';

export { hasContext };
