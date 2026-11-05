/* @layer renderer-app @kind logic */
import type { SignalItem } from '@drizztdourden08/tessera';
import type { TestsView } from './model.type';

const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

const summarySignals = (view: TestsView): SignalItem[] => {
  const items: SignalItem[] = [
    { id: 'runs', icon: 'flag', label: plural(view.runs.length, 'run open', 'runs open'), tone: 'info' },
    { id: 'to-run', icon: 'list-checks', label: `${view.toRun} of ${plural(view.total, 'case', 'cases')} to run`, tone: view.toRun > 0 ? 'warning' : 'neutral' },
  ];
  if (view.failed > 0) items.push({ id: 'failed', icon: 'circle-x', label: `${view.failed} failed`, tone: 'danger' });
  if (view.passed > 0) items.push({ id: 'passed', icon: 'circle-check', label: `${view.passed} passed`, tone: 'success' });
  if (view.filtered) items.push({ id: 'filtered', icon: 'funnel', label: 'filtered', tone: 'neutral' });
  return items;
};

export { summarySignals };
