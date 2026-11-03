/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../../data/runs';
import { recordMenuGroups, useRecordPane } from '../../../../../shell';
import type { RecordHeadProps } from '../../../../../shell/records';
import { usePlay } from '../../../run-play';
import { copyText } from '../../../shared';
import type { GroupThreads } from '../../../item-groups';
import { BUILD_KIND } from '../builds.constants';
import { buildCode } from '../model';
import type { LauncherView } from '../model';

const useBuildHead = (view: LauncherView, boardKey: string, threads: GroupThreads): RecordHeadProps => {
  const { launcher, def } = view;
  const pane = useRecordPane();
  const play = usePlay(boardKey);
  const code = buildCode(launcher.name);
  const canPlay = view.playable && def !== null && !view.runs.some(isLiveRun);
  const handlers = {
    open: () => pane.open(code),
    play: canPlay ? () => void play.start(def, launcher.name) : undefined,
    'copy-command': launcher.cmd ? () => copyText(launcher.cmd ?? '') : undefined,
    copy: () => copyText(code),
  };
  return {
    kind: BUILD_KIND,
    code,
    assignee: launcher.by ?? launcher.createdBy ?? '',
    boardKey,
    threads,
    menu: recordMenuGroups(BUILD_KIND, code, handlers),
    menuLabel: `More actions for ${code}`,
  };
};

export { useBuildHead };
