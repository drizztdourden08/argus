/* @layer renderer-app @kind logic */
import { STALL_HEAD } from './stalls.constants';

const stallText = (text: string): string => (text.startsWith(STALL_HEAD) ? text.slice(STALL_HEAD.length) : text);

export { stallText };
