/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../../data/runs';
import { recordMenuGroups, useRecordPane } from '../../../../../shell';
import type { RecordHeadProps } from '../../../../../shell/records';
import { usePlay } from '../../../run-play';
import { copyText } from '../../../shared';
import type { GroupThreads } from '../../../item-groups';
import type { OwnerAsk } from '../../../owner-links';
import { BUILD_KIND } from '../builds.constants';
import { buildCode, isMaster } from '../model';
import type { LauncherView } from '../model';

const useBuildHead = (view: LauncherView, boardKey: string, threads: GroupThreads, ask: (ask: OwnerAsk) => void): RecordHeadProps => {
  const { launcher, def } = view;
  const pane = useRecordPane();
  const play = usePlay(boardKey);
  const code = buildCode(launcher.name);
  const canPlay = view.playable && def !== null && !view.runs.some(isLiveRun);
  const live = !launcher.retiredAt;
  const owner = (action: OwnerAsk['action']) => () => ask({ action, id: launcher.name, label: code });
  const handlers = {
    open: () => pane.open(code),
    play: canPlay ? () => void play.start(def, launcher.name) : undefined,
    'copy-command': launcher.cmd ? () => copyText(launcher.cmd ?? '') : undefined,
    link: live ? owner('launcher-link') : undefined,
    retire: live && !isMaster(launcher) ? owner('launcher-retire') : undefined,
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
