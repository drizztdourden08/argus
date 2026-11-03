/* @layer renderer-app @kind logic */
import type { ReleasePr, ReleaseVersion } from '../../../data/remote';
import { COMMIT_MATCH_MIN } from '../releases.constants';
import type { PullRequestDoc, Shipped, ShippedSource } from '../releases.type';

const sameCommit = (a: string, b: string | null | undefined): boolean => {
  if (!b || a.length < COMMIT_MATCH_MIN || b.length < COMMIT_MATCH_MIN) return false;
  return a.startsWith(b) || b.startsWith(a);
};

const localOf = (pr: ReleasePr, prs: readonly PullRequestDoc[]): PullRequestDoc | null =>
  prs.find((candidate) => sameCommit(pr.commit, candidate.mergeCommit)) ?? null;

const shippedOf = (version: ReleaseVersion, source: ShippedSource): Shipped => {
  const ids = version.items ?? [];
  const known = new Map(source.items.map((item) => [item.id, item]));
  return {
    items: ids.flatMap((id) => {
      const item = known.get(id);
      return item ? [item] : [];
    }),
    unknown: ids.filter((id) => !known.has(id)),
    prs: (version.prs ?? []).map((pr) => ({ pr, local: localOf(pr, source.prs) })),
  };
};

export { shippedOf };
