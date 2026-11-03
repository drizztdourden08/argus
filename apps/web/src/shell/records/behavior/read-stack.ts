/* @layer renderer-app @kind logic */
import { NO_STACK, STACK_KEY } from '../records.constants';

const isCodes = (value: unknown): value is string[] => Array.isArray(value) && value.every((entry) => typeof entry === 'string');

const readStack = (state: unknown): readonly string[] => {
  if (typeof state !== 'object' || state === null) return NO_STACK;
  const stack: unknown = Reflect.get(state, STACK_KEY);
  return isCodes(stack) ? stack : NO_STACK;
};

export { readStack };
