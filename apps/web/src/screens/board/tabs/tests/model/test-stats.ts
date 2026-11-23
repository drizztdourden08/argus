/* @layer renderer-app @kind logic */
import type { TabStat } from '../../../shared';
import type { TestsView } from './model.type';

const testStats = (view: TestsView): TabStat[] => [
  { id: 'runs', label: 'runs open', count: view.runs.length, tone: 'info' },
  { id: 'to-run', label: 'to run', count: view.toRun, tone: view.toRun > 0 ? 'warning' : 'neutral' },
  { id: 'failed', label: 'failed', count: view.failed, tone: view.failed > 0 ? 'danger' : 'neutral' },
  { id: 'passed', label: 'passed', count: view.passed, tone: view.passed > 0 ? 'success' : 'neutral' },
];

export { testStats };
