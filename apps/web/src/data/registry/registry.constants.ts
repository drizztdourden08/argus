/* @layer renderer-app @kind constants */
import registryFile from '@argus/registry/item-types.json';
import { parseRegistry } from './parse-registry';

const ITEM_REGISTRY = parseRegistry(registryFile);

export { ITEM_REGISTRY };
