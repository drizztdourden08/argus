/* @layer renderer-app @kind test */
const minutesAgo = (minutes: number): string => new Date(Date.now() - minutes * 60000).toISOString();

const CARDS = {
  rotp: { key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/' },
  argus: { key: 'argus', name: 'Argus', state: 'running', root: null, url: '/p/argus/' },
};

const PROCS = {
  '4812-1': {
    pid: 4812, name: 'vite', cmd: 'node vite --port 5173', rule: 'node-dev-server', project: 'rotp', memMB: 1900, cpuPct: 0.2, startedAt: minutesAgo(190),
    ageMin: 190, idleMin: 180, status: 'stale', flags: ['stale'], protected: false, killable: true,
  },
  '4900-1': {
    pid: 4900, name: 'electron', cmd: 'electron .', rule: 'electron-dev', project: null, memMB: 1600, cpuPct: 12, startedAt: minutesAgo(60),
    ageMin: 60, idleMin: 0, status: 'hog', flags: ['hog'], protected: false, killable: true,
  },
  '100-1': {
    pid: 100, name: 'explorer', cmd: 'explorer.exe', rule: 'shell-core', project: null, memMB: 800, cpuPct: 1, startedAt: minutesAgo(600),
    ageMin: 600, idleMin: 0, status: 'ok', flags: [], protected: true, protectedWhy: 'system', killable: false,
  },
};

const PROCS_META = {
  procs: {
    sampledAt: minutesAgo(1),
    count: 3,
    totalMemMB: 4300,
    platform: 'win32',
    error: null,
    byRule: [
      { rule: 'node-dev-server', count: 1, matched: 1, memMB: 1900 },
      { rule: 'electron-dev', count: 1, matched: 1, memMB: 1600 },
      { rule: 'shell-core', count: 1, matched: 1, memMB: 800 },
    ],
    rules: [
      { name: 'node-dev-server', match: { name: '^node' }, maxMemMB: 1500, maxGroupMemMB: 1500, idleMin: 180, orphan: true, killable: true, note: 'dev servers' },
      { name: 'electron-dev', match: { name: '^electron' }, maxMemMB: 1500, killable: true },
      { name: 'shell-core', match: { name: '^explorer' }, killable: false },
    ],
  },
};

const ACTIVITY = {
  'a-1': { n: 1, at: minutesAgo(5), kind: 'run', level: 'error', text: 'R-3 play failed (exit 1)', project: 'rotp', thread: 'lead' },
  'a-2': { n: 2, at: minutesAgo(3), kind: 'note', level: 'info', text: 'ping from rotp/lead: build ready', project: 'rotp', thread: 'lead' },
  'a-3': { n: 3, at: minutesAgo(2), kind: 'fault', level: 'warn', text: 'rotp/lead: its wake listener is not running', project: 'rotp', thread: 'lead' },
};

const NOTIFICATIONS = {
  'n-2': { n: 2, at: minutesAgo(1), level: 'urgent', title: 'RotP: build ready', text: 'Build is ready to test', project: 'rotp', thread: 'lead', by: 'auto', toast: 'shown', url: null, why: null },
  'n-1': { n: 1, at: minutesAgo(9), level: 'info', title: 'Argus', text: 'FYI', project: null, thread: null, by: 'page', toast: 'skipped', url: null, why: 'info is recorded only' },
};

export { ACTIVITY, CARDS, minutesAgo, NOTIFICATIONS, PROCS, PROCS_META };
