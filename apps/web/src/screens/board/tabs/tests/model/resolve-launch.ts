/* @layer renderer-app @kind logic */
import type { BuildDoc, LaunchDefDoc, LauncherDoc, LaunchInfo } from './model.type';

const NONE: LaunchInfo = { state: 'none', name: null, cmd: '', cwd: '', def: null, gone: '' };

const retiredText = (launcher: LauncherDoc): string => {
  const by = launcher.retiredBy ? ` by ${launcher.retiredBy}` : '';
  const note = launcher.retireNote ? ` (${launcher.retireNote})` : '';
  return `launcher ${launcher.name} was retired${by}${note}`;
};

const linked = (name: string, launcher: LauncherDoc | undefined, defs: readonly LaunchDefDoc[]): LaunchInfo => {
  if (!launcher) return { ...NONE, state: 'missing', name, gone: `launcher ${name} is missing: the board has no launcher by that name` };
  if (launcher.retiredAt) return { ...NONE, state: 'retired', name, gone: retiredText(launcher) };
  const def = defs.find((candidate) => candidate.launcher === name) ?? null;
  return { state: 'linked', name, cmd: launcher.cmd ?? '', cwd: launcher.cwd ?? '', def, gone: '' };
};

const resolveLaunch = (build: BuildDoc | null, launchers: readonly LauncherDoc[], defs: readonly LaunchDefDoc[]): LaunchInfo => {
  if (build?.launcher) return linked(build.launcher, launchers.find((candidate) => candidate.name === build.launcher), defs);
  if (!build?.cmd) return NONE;
  const def = defs.find((candidate) => candidate.bundle === build.id) ?? null;
  return { state: 'legacy', name: null, cmd: build.cmd, cwd: build.cwd ?? '', def, gone: '' };
};

export { resolveLaunch };
