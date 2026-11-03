/* @layer renderer-app @kind logic */
import type { Rejected, RefusalKey } from './content.type';

const reject = (key: RefusalKey, message: string): Rejected => ({ ok: false, key, message });

export { reject };
