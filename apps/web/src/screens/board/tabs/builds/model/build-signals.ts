/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../../data/runs';
import { isMaster } from './is-master';
import type { BuildSignal, LauncherView } from './model.type';

const buildSignals = (view: LauncherView): BuildSignal[] => {
  const { launcher } = view;
  if (launcher.retiredAt) return [];
  const found: BuildSignal[] = [];
  if (view.runs.some(isLiveRun)) found.push({ id: 'live', icon: 'play', label: 'Running now', tone: 'info', tab: 'runs' });
  if (view.usage?.stale) found.push({ id: 'stale', icon: 'clock', label: 'Stale: not used within its window', tone: 'warning', tab: 'overview' });
  if (launcher.relink) found.push({ id: 'relink', icon: 'link', label: 'Needs a new link to the item it serves', tone: 'warning', tab: 'links' });
  else if (!launcher.for && !isMaster(launcher)) found.push({ id: 'no-item', icon: 'link', label: 'Serves no item yet: link it', tone: 'warning', tab: 'links' });
  return found;
};

export { buildSignals };
