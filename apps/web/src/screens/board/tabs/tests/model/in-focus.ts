/* @layer renderer-app @kind logic */
import type { FocusScope, TestDoc } from './model.type';

const inFocus = (tests: readonly TestDoc[], scope: FocusScope): TestDoc[] => {
  const { focus, paused, showPaused } = scope;
  return tests.filter((test) => {
    const thread = test.thread ?? '';
    return (!thread || showPaused || !paused.has(thread)) && (!focus || thread === focus);
  });
};

export { inFocus };
