/* @layer renderer-app @kind logic */
import { kindByCode } from '../../../data/registry';
import type { RegistryKind } from '../../../data/registry';
import { WRITTEN } from './resolve-code.constants';

interface TypedItemCode {
  kind: RegistryKind;
  number: string;
}

const typedItemCode = (code: string): TypedItemCode | null => {
  const found = WRITTEN.exec(code);
  const kind = found ? kindByCode(found[1] ?? '') : undefined;
  if (!found || !kind || kind.family !== 'type' || kind.collection !== 'items') return null;
  return { kind, number: found[2] ?? '' };
};

export { typedItemCode };
