/* @layer renderer-app @kind logic */
import type { ItemRegistry } from './registry.type';

const parseRegistry = (raw: unknown): ItemRegistry => raw as ItemRegistry;

export { parseRegistry };
