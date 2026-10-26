/* @layer renderer-app @kind test */
const NOW = '2026-10-26T10:00:00.000Z';
const EARLIER = '2026-10-26T09:00:00.000Z';

const testDoc = (id: string, extra: object = {}) => ({
  id,
  item: 'I-1',
  thread: 'lead',
  title: `Test ${id}`,
  covers: null,
  build: 'bundle-1',
  mode: 'manual',
  run: null,
  context: null,
  changed: [],
  setup: [],
  steps: [],
  pass: [],
  fail: [],
  notes: [],
  ac: [],
  status: 'pending',
  result: null,
  runs: [],
  proofs: [],
  rank: 1,
  createdAt: EARLIER,
  updatedAt: EARLIER,
  ...extra,
});

const doneDoc = (id: string, status: string, text: string, extra: object = {}) => {
  const result = { status, text, by: 'human', at: NOW };
  return testDoc(id, { status, result, runs: [result], updatedAt: NOW, ...extra });
};

const buildDoc = (tag: string, extra: object = {}) => ({
  tag,
  status: 'ready' as 'draft' | 'ready',
  title: null,
  note: null,
  launcher: null,
  cmd: null,
  cwd: null,
  thread: 'lead',
  updatedAt: EARLIER,
  closedAt: null,
  closedBy: null,
  closeNote: null,
  ...extra,
});

const launcherDoc = (name: string, extra: object = {}) => ({
  name,
  cmd: `rotp launch ${name}`,
  cwd: null,
  retiredAt: null,
  retiredBy: null,
  retireNote: null,
  ...extra,
});

const defDoc = (launcher: string, extra: object = {}) => ({
  name: `launcher-${launcher}`,
  launcher,
  cmd: `rotp launch ${launcher}`,
  hand: true,
  window: false,
  ...extra,
});

const runDoc = (id: string, name: string, extra: object = {}) => ({
  id,
  name,
  status: 'exited',
  exitCode: 0,
  queuedAt: EARLIER,
  startedAt: EARLIER,
  endedAt: NOW,
  error: null,
  lastLine: null,
  ...extra,
});

const byId = (docs: Record<string, unknown>[], key = 'id'): Record<string, object> => Object.fromEntries(docs.map((doc) => [String(doc[key]), doc]));

export { buildDoc, byId, defDoc, doneDoc, launcherDoc, runDoc, testDoc };
