/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../../data/runs';
import { recordMenuGroups, useRecordPane } from '../../../../../shell';
import type { RecordHeadProps } from '../../../../../shell/records';
import { usePlay } from '../../../run-play';
import { copyText } from '../../../shared';
import type { GroupThreads } from '../../../item-groups';
import { withUnlink } from '../../../owner-links';
import type { OwnerAsk } from '../../../owner-links';
import { BUILD_KIND } from '../builds.constants';
import { buildCode, isMaster } from '../model';
import type { LauncherView } from '../model';
import type { Play } from './useBuildHead.type';

const playHandler = (view: LauncherView, play: Play): (() => void) | undefined => {
  const { launcher, def } = view;
  const canPlay = view.playable && def !== null && !view.runs.some(isLiveRun);
  return canPlay ? () => void play.start(def, launcher.name) : undefined;
};

const commandHandler = (launcher: LauncherView['launcher']): (() => void) | undefined => {
  const cmd = launcher.cmd;
  return cmd ? () => copyText(cmd) : undefined;
};

const ownerHandlers = (launcher: LauncherView['launcher'], code: string, ask: (ask: OwnerAsk) => void) => {
  const live = !launcher.retiredAt;
  const owner = (action: OwnerAsk['action']) => () => ask({ action, id: launcher.name, label: code });
  const master = isMaster(launcher);
  return {
    link: live ? owner('launcher-link') : undefined,
    retire: live && !master ? owner('launcher-retire') : undefined,
    unlink: live && master && Boolean(launcher.for) ? owner('launcher-unlink') : undefined,
  };
};

const useBuildHead = (view: LauncherView, boardKey: string, threads: GroupThreads, ask: (ask: OwnerAsk) => void): RecordHeadProps => {
  const { launcher } = view;
  const pane = useRecordPane();
  const play = usePlay(boardKey);
  const code = buildCode(launcher.name);
  const { link, retire, unlink } = ownerHandlers(launcher, code, ask);
  const handlers = {
    open: () => pane.open(code),
    play: playHandler(view, play),
    'copy-command': commandHandler(launcher),
    link,
    retire,
    copy: () => copyText(code),
  };
  return {
    kind: BUILD_KIND,
    code,
    assignee: launcher.by ?? launcher.createdBy ?? '',
    boardKey,
    threads,
    menu: withUnlink(recordMenuGroups(BUILD_KIND, code, handlers), unlink),
    menuLabel: `More actions for ${code}`,
  };
};

export { useBuildHead };
