/* @layer renderer-app @kind test */
import { exitedRun, launchDef } from '../helpers/launch-docs';
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

const defDoc = (launcher: string, extra: object = {}) => launchDef(launcher, `rotp launch ${launcher}`, extra);

const runDoc = (id: string, name: string, extra: object = {}) => exitedRun(id, name, { queuedAt: EARLIER, startedAt: EARLIER, endedAt: NOW }, extra);

const itemDoc = (id: string, extra: object = {}) => ({
  id,
  title: `Item ${id}`,
  thread: 'lead',
  subject: 'work',
  status: 'doing',
  rev: 2,
  ac: [],
  createdAt: EARLIER,
  updatedAt: EARLIER,
  ...extra,
});

const threadDoc = (id: string, extra: object = {}) => ({ id, name: id, session: id, role: 'thread', status: 'active', ...extra });

const wakeDoc = (id: string, subject: string, extra: object = {}) => ({
  id,
  subject,
  thread: 'lead',
  status: 'delivered',
  listeners: 1,
  at: NOW,
  ...extra,
});

const conflictEntry = (id: string, test: string, extra: object = {}) => ({
  id,
  kind: 'move',
  author: 'human',
  target: { c: 'tests', id: test },
  to: { item: 'I-2' },
  prev: { item: 'I-1' },
  baseRev: 2,
  status: 'conflict',
  by: 'lead',
  current: { item: 'I-3' },
  createdAt: EARLIER,
  ...extra,
});

const byId = (docs: Record<string, unknown>[], key = 'id'): Record<string, object> => Object.fromEntries(docs.map((doc) => [String(doc[key]), doc]));

export { buildDoc, byId, conflictEntry, defDoc, doneDoc, itemDoc, launcherDoc, runDoc, testDoc, threadDoc, wakeDoc };
