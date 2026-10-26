/* @layer renderer-app @kind test */
interface RunTimes {
  queuedAt: string;
  startedAt: string;
  endedAt: string;
}

const launchDef = (launcher: string, cmd: string, extra: object = {}) => ({
  name: `launcher-${launcher}`,
  launcher,
  cmd,
  hand: true,
  window: false,
  ...extra,
});

const exitedRun = (id: string, name: string, times: RunTimes, extra: object = {}) => ({
  id,
  name,
  status: 'exited',
  exitCode: 0,
  ...times,
  error: null,
  lastLine: null,
  ...extra,
});

export { exitedRun, launchDef };
