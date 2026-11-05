/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';
import type { RunEntry } from '../../../../../data/runs';
import { itemCode, recordMenuGroups, useRecordPane } from '../../../../../shell';
import { copyText } from '../../../shared';
import { shownRun } from '../model';
import type { RunView } from '../model';
import { BUILD_KIND, RUN_KIND } from '../tests.constants';
import { useTestsShared } from './useTestsShared';

const useRunMenu = (view: RunView, runs: readonly RunEntry[]): MenuGroup[] => {
  const { onLog } = useTestsShared();
  const pane = useRecordPane();
  const code = itemCode(view.item);
  const shown = view.stage === 'draft' ? null : shownRun(view.launch, runs);
  const launch: MenuItem[] = [];
  if (view.build) {
    const build = itemCode({ id: view.build, type: BUILD_KIND });
    launch.push({ id: 'build', label: 'Open build', onSelect: () => pane.open(build, 'runs') });
  }
  if (shown) launch.push({ id: 'log', label: `Log of ${shown.id}`, onSelect: () => onLog(shown.id) });
  const middle: MenuGroup[] = launch.length > 0 ? [{ id: 'launch', items: launch }] : [];
  return recordMenuGroups(RUN_KIND, code, { open: () => pane.open(code), copy: () => copyText(code) }, middle);
};

export { useRunMenu };
