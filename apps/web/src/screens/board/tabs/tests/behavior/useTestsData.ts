/* @layer renderer-app @kind logic */
import type { Entry, ThreadChoice } from '../../../../../data/kanban';
import { useCollection, useDoc } from '../../../../../data/react';
import type { BoardMeta, Build, LaunchDef, Launcher, RunDoc, Test, TestItem, Wake } from '../../../../../data/types';
import type { TestsData } from '../tests.type';

const useTestsData = (boardKey: string): TestsData => {
  const tests = useCollection<Test>(boardKey, 'tests');
  const builds = useCollection<Build>(boardKey, 'builds');
  const launchers = useCollection<Launcher>(boardKey, 'launchers');
  const defs = useCollection<LaunchDef>(boardKey, 'launches');
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const items = useCollection<TestItem>(boardKey, 'items');
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const entries = useCollection<Entry>(boardKey, 'entries');
  const wakes = useCollection<Wake>(boardKey, 'wakes');
  const meta = useDoc<BoardMeta>(boardKey, 'meta', 'board');
  return {
    ready: tests.ready && builds.ready,
    tests: tests.docs,
    builds: builds.docs,
    launchers: launchers.docs,
    defs: defs.docs,
    runs: runs.docs,
    items: items.docs,
    threads: threads.docs,
    entries: entries.docs,
    wakes: wakes.docs,
    handed: meta.doc?.handedBuild ?? null,
  };
};

export { useTestsData };
