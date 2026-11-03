/* @layer renderer-app @kind logic */
import type { CodeEntry, RegistryKind } from './registry.type';

const byLongestFirst = (a: CodeEntry, b: CodeEntry): number => b.letters.length - a.letters.length || a.letters.localeCompare(b.letters);

const entriesOf = (kind: RegistryKind): CodeEntry[] => [kind.code, ...kind.aliases].map((letters) => ({ letters, kind: kind.id }));

const buildCodeTable = (kinds: readonly RegistryKind[]): CodeEntry[] =>
  kinds
    .filter((kind) => kind.shares === undefined)
    .flatMap(entriesOf)
    .sort(byLongestFirst);

export { buildCodeTable };
