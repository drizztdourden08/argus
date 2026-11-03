/* @layer renderer-app @kind logic */
import type { PaneBadge, PaneData } from '../pane.type';

const testsBadge = (data: PaneData): PaneBadge | undefined => {
  const total = data.tests.length;
  if (total === 0) return undefined;
  const failing = data.tests.filter((test) => test.status === 'fail').length;
  return { value: `${failing}/${total}`, tone: failing > 0 ? 'danger' : 'neutral', title: `${failing} failing of ${total} tests` };
};

export { testsBadge };
