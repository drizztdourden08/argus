/* @layer renderer-app @kind test */
import { exitedRun, launchDef } from '../helpers/launch-docs';
const MIN = 60 * 1000;
const DAY = 24 * 60 * MIN;

const ago = (ms: number): string => new Date(Date.now() - ms).toISOString();

const launcherDoc = (name: string, extra: object = {}) => ({
  name,
  title: `Launcher ${name}`,
  cmd: `game run --${name}`,
  cwd: null,
  kind: 'app',
  branch: null,
  commit: null,
  note: null,
  current: false,
  window: false,
  bundle: null,
  retiredAt: null,
  retiredBy: null,
  retireNote: null,
  createdBy: 'coordinator',
  updatedAt: ago(2 * MIN),
  by: 'coordinator',
  ...extra,
});

const standaloneDoc = (name: string, extra: object = {}) =>
  launcherDoc(name, { standalone: true, joined: null, useEvery: '7d', useEveryMs: 7 * DAY, lastUsedAt: ago(DAY), keptUntil: null, ...extra });

const defDoc = (launcher: string, extra: object = {}) => launchDef(launcher, `game run --${launcher}`, extra);

const runDoc = (id: string, name: string, extra: object = {}) =>
  exitedRun(id, name, { queuedAt: ago(5 * MIN), startedAt: ago(5 * MIN), endedAt: ago(4 * MIN) }, { stdin: true, ...extra });

const runningDoc = (id: string, name: string, extra: object = {}) => runDoc(id, name, { status: 'running', endedAt: null, exitCode: null, ...extra });

const buildDoc = (tag: string, extra: object = {}) => ({
  tag,
  status: 'ready',
  title: null,
  note: null,
  launcher: null,
  cmd: null,
  cwd: null,
  thread: 'lead',
  updatedAt: ago(DAY),
  closedAt: null,
  closedBy: null,
  closeNote: null,
  ...extra,
});

export { ago, buildDoc, DAY, defDoc, launcherDoc, MIN, runDoc, runningDoc, standaloneDoc };
