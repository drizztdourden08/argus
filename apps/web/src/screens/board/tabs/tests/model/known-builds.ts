/* @layer renderer-app @kind logic */
import { compareIds } from './compare-ids';
import type { BuildDoc, TestDoc } from './model.type';

const knownBuilds = (tests: readonly TestDoc[], builds: readonly BuildDoc[], handed: string | null, showClosed: boolean): string[] => {
  const closed = new Set(builds.filter((build) => build.closedAt).map((build) => build.id));
  const shown = (tag: string): boolean => showClosed || !closed.has(tag);
  const tags = new Set(tests.filter((test) => !test.deletedAt && test.build && shown(test.build)).map((test) => String(test.build)));
  for (const build of builds) if (shown(build.id)) tags.add(build.id);
  if (handed) tags.add(handed);
  return [...tags].sort((a, b) => Number(b === handed) - Number(a === handed) || compareIds(b, a));
};

export { knownBuilds };
