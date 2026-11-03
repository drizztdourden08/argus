/* @layer renderer-app @kind constants */
import { buildCodeTable } from './build-code-table';
import { listKinds } from './list-kinds';
import { ITEM_REGISTRY } from './registry.constants';

const REGISTRY_KINDS = listKinds(ITEM_REGISTRY);

const CODE_TABLE = buildCodeTable(REGISTRY_KINDS);

export { CODE_TABLE, REGISTRY_KINDS };
