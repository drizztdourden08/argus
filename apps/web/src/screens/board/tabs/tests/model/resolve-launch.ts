/* @layer renderer-app @kind logic */
import type { LaunchDefDoc, LauncherDoc, LaunchInfo } from './model.type';

const NONE: LaunchInfo = { state: 'none', name: null, cmd: '', cwd: '', def: null, gone: '', note: '', updatedAt: '' };

const retiredText = (launcher: LauncherDoc): string => {
  const by = launcher.retiredBy ? ` by ${launcher.retiredBy}` : '';
  const note = launcher.retireNote ? ` (${launcher.retireNote})` : '';
  return `launcher ${launcher.name} was retired${by}${note}`;
};

const linked = (name: string, launcher: LauncherDoc | undefined, defs: readonly LaunchDefDoc[]): LaunchInfo => {
  if (!launcher) return { ...NONE, state: 'missing', name, gone: `launcher ${name} is missing: the board has no launcher by that name` };
  if (launcher.retiredAt) return { ...NONE, state: 'retired', name, gone: retiredText(launcher) };
  const def = defs.find((candidate) => candidate.launcher === name) ?? null;
  return { state: 'linked', name, cmd: launcher.cmd ?? '', cwd: launcher.cwd ?? '', def, gone: '', note: launcher.note ?? '', updatedAt: launcher.updatedAt ?? '' };
};

const resolveLaunch = (build: string | null, launchers: readonly LauncherDoc[], defs: readonly LaunchDefDoc[]): LaunchInfo =>
  build ? linked(build, launchers.find((candidate) => candidate.name === build), defs) : NONE;

export { resolveLaunch };
