/* @layer renderer-app @kind logic */
import type { RegistryKind } from '../../../data/registry';

const shownCode = (kind: RegistryKind, id: string): string => {
  if (kind.idKey !== undefined) return `${kind.code}-${id}`;
  const stored = kind.stored;
  return stored && id.startsWith(`${stored}-`) ? `${kind.code}-${id.slice(stored.length + 1)}` : id;
};

export { shownCode };
