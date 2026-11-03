/* @layer renderer-app @kind logic */
import { STACK_KEY } from '../records.constants';

const stackState = (stack: readonly string[]): Record<string, readonly string[]> | undefined => (stack.length > 0 ? { [STACK_KEY]: stack } : undefined);

export { stackState };
