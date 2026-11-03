/* @layer renderer-app @kind types */
import type { RegistryKind } from '../../../data/registry';

interface TypedItemCode {
  kind: RegistryKind;
  number: string;
}

export type { TypedItemCode };
