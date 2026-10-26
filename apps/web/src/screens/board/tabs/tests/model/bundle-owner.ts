/* @layer renderer-app @kind logic */
import type { BundleGroup } from './model.type';

const bundleOwner = (group: BundleGroup): string => {
  if (group.build?.thread) return group.build.thread;
  const threads = [...new Set(group.tests.map((test) => test.thread).filter(Boolean))];
  return threads.length === 1 ? String(threads[0]) : '';
};

export { bundleOwner };
