/* @layer renderer-app @kind logic */
import { FOCUS_KEY } from '../shared.constants';

const focusKey = (boardKey: string): string => `${FOCUS_KEY}.${boardKey}`;

export { focusKey };
