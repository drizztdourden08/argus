/* @layer renderer-app @kind logic */
import { kindOf } from './kind-of';
import { CODE_TABLE } from './kinds.constants';
import type { RegistryKind } from './registry.type';

const kindByCode = (letters: string): RegistryKind | undefined => {
  const entry = CODE_TABLE.find((candidate) => candidate.letters === letters);
  return entry ? kindOf(entry.kind) : undefined;
};

export { kindByCode };
