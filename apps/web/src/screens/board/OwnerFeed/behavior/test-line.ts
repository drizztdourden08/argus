/* @layer renderer-app @kind logic */
import type { TestLineProps, TestLineView } from '../OwnerFeed.type';

const testLine = (card: TestLineProps['card']): TestLineView => {
  if (card.type === 'test') {
    const { build, title, thread, id } = card.data;
    return { build, count: '', title, hover: title, thread, open: id };
  }
  const { tests, build, thread } = card.data;
  const first = tests[0];
  return {
    build,
    count: `${tests.length} ${tests.length > 1 ? 'tests' : 'test'} to run`,
    title: `${first?.title ?? ''}${tests.length > 1 ? ' ...' : ''}`,
    hover: tests.map((test) => test.title).join('\n'),
    thread,
    open: first?.id ?? 'tests',
  };
};

export { testLine };
