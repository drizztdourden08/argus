/* @layer renderer-app @kind logic */
import { isLive } from '../../../../data/deps';
import type { DepDoc, DepEnd } from '../../../../data/deps';
import type { WithId } from '../../../../data/react';

type Dep = WithId<DepDoc>;

const keyOf = (end: DepEnd): string => `${end.project}/${end.item ?? ''}`;

const nextOf = (tip: Dep, docs: readonly Dep[], seen: ReadonlySet<string>): Dep | undefined =>
  docs.find((doc) => !seen.has(doc.id) && isLive(doc.state) && doc.from.item !== null && keyOf(doc.from) === keyOf(tip.to));

const walk = (tip: Dep, docs: readonly Dep[], seen: ReadonlySet<string>): number => {
  const next = nextOf(tip, docs, seen);
  return next ? 1 + walk(next, docs, new Set([...seen, next.id])) : 0;
};

const trailLength = (doc: Dep, docs: readonly Dep[]): number => 1 + walk(doc, docs, new Set([doc.id]));

export { trailLength };
