/* @layer renderer-app @kind logic */
import { bundleTag } from './bundle-tag';
import { compareIds } from './compare-ids';
import { isToRun } from './is-to-run';
import { NO_BUNDLE } from './model.constants';
import type { BuildDoc, BundleGroup, GroupInput, TestDoc } from './model.type';
import { tally } from './tally';

const rank = (group: BundleGroup): number => {
  if (group.handed) return 0;
  if (group.tag === NO_BUNDLE) return 3;
  return group.counts.run > 0 ? 1 : 2;
};

const byRank = (a: BundleGroup, b: BundleGroup): number => rank(a) - rank(b) || compareIds(b.tag, a.tag);

const openTests = (tests: readonly TestDoc[], closed: ReadonlySet<string>): TestDoc[] =>
  tests.filter((test) => !(test.build && closed.has(test.build)));

const declaredTags = (builds: readonly BuildDoc[], handed: string | null): string[] => {
  const tags = builds.filter((build) => !build.closedAt).map((build) => build.id);
  return handed && !tags.includes(handed) ? [...tags, handed] : tags;
};

const groupBundles = (input: GroupInput): BundleGroup[] => {
  const { tests, builds, handed } = input;
  const closed = new Set(builds.filter((build) => build.closedAt).map((build) => build.id));
  const slots = new Map<string, TestDoc[]>(declaredTags(builds, handed).map((tag) => [tag, []]));
  for (const test of openTests(tests, closed)) {
    const tag = bundleTag(test);
    slots.set(tag, [...(slots.get(tag) ?? []), test]);
  }
  const groups = [...slots].map(([tag, rows]): BundleGroup => {
    const counts = tally(rows);
    const isHanded = tag === handed;
    return { tag, build: builds.find((build) => build.id === tag) ?? null, tests: rows, counts, handed: isHanded, open: isHanded || rows.some(isToRun) };
  });
  return groups.sort(byRank);
};

export { groupBundles };
