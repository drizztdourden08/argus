/* @layer renderer-app @kind logic */
import type { RegistryKind } from '../../../data/registry';
import { fieldKeys } from './field-keys';

const groupKeysOf = (kind: RegistryKind, doc: Readonly<Record<string, unknown>>): string[] => fieldKeys(doc, kind.groupBy);

export { groupKeysOf };
