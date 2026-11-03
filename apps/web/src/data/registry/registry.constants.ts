/* @layer renderer-app @kind constants */
import registryFile from '@argus/registry/item-types.json';
import { parseRegistry } from './parse-registry';

const ITEM_REGISTRY = parseRegistry(registryFile);

const ITEM_STORED = 'I';

export { ITEM_REGISTRY, ITEM_STORED };
