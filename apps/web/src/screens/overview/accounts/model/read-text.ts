/* @layer renderer-app @kind logic */
import { clockText } from './clock-text';

const readText = (readAt: string | null | undefined): string => clockText(readAt) || 'Not read yet';

export { readText };
