/* @layer renderer-app @kind logic */
import { REGISTRY_KINDS } from './kinds.constants';
import type { RegistryKind } from './kind.type';

const kindOf = (id: string): RegistryKind | undefined => REGISTRY_KINDS.find((kind) => kind.id === id);

export { kindOf };
