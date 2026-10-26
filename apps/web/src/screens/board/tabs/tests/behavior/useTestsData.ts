/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection, useDoc } from '../../../../../data/react';
import type { Build, BoardMeta, LaunchDef, Launcher, RunDoc, Test, TestItem } from '../../../../../data/types';
import { closedBundles, groupBundles, resolveLaunch, visibleTests } from '../model';
import type { TestsData } from '../tests.type';

const useTestsData = (boardKey: string): TestsData => {
  const tests = useCollection<Test>(boardKey, 'tests');
  const builds = useCollection<Build>(boardKey, 'builds');
  const launchers = useCollection<Launcher>(boardKey, 'launchers');
  const defs = useCollection<LaunchDef>(boardKey, 'launches');
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const items = useCollection<TestItem>(boardKey, 'items');
  const meta = useDoc<BoardMeta>(boardKey, 'meta', 'board');
  const ready = tests.ready && builds.ready;
  const handed = meta.doc?.handedBuild ?? null;
  const visible = useMemo(() => visibleTests(tests.docs), [tests.docs]);
  const bundles = useMemo(
    () => groupBundles({ tests: visible, builds: builds.docs, handed }).map((group) => ({ group, launch: resolveLaunch(group.build, launchers.docs, defs.docs) })),
    [visible, builds.docs, handed, launchers.docs, defs.docs],
  );
  const closed = useMemo(() => closedBundles(visible, builds.docs), [visible, builds.docs]);
  return { ready, tests: visible, bundles, closed, runs: runs.docs, items: items.docs };
};

export { useTestsData };
