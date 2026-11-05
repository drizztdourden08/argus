/* @layer renderer-app @kind test */
import type { RunEntry } from '../../src/data/runs';
import type { TestDoc } from '../../src/screens/board/tabs/tests/model';
import { exitedRun, launchDef } from '../helpers/launch-docs';
type Doc = Record<string, unknown>;

const NOW = '2026-10-26T10:00:00.000Z';
const EARLIER = '2026-10-26T09:00:00.000Z';

const testDoc = (id: string, extra: object = {}): Doc => ({
  id,
  item: 'I-1',
  thread: 'lead',
  title: `Test number ${id.slice(2)}`,
  covers: null,
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

const doneDoc = (id: string, status: string, text: string, extra: object = {}): Doc => {
  const result = { status, text, by: 'human', at: NOW };
  return testDoc(id, { status, result, runs: [result], updatedAt: NOW, ...extra });
};

const launcherDoc = (name: string, extra: object = {}): Doc => ({
  name,
  cmd: `rotp launch ${name}`,
  cwd: null,
  retiredAt: null,
  retiredBy: null,
  retireNote: null,
  ...extra,
});

const defDoc = (launcher: string, extra: object = {}): Doc => launchDef(launcher, `rotp launch ${launcher}`, extra);

const runDoc = (id: string, name: string, extra: object = {}): Doc => exitedRun(id, name, { queuedAt: EARLIER, startedAt: EARLIER, endedAt: NOW }, extra);

const itemDoc = (id: string, extra: object = {}): Doc => ({
  id,
  title: `Item number ${id.slice(2)}`,
  thread: 'lead',
  subject: 'work',
  status: 'doing',
  rev: 2,
  ac: [],
  createdAt: EARLIER,
  updatedAt: EARLIER,
  ...extra,
});

const testRunDoc = (id: string, plan: string, extra: object = {}): Doc =>
  itemDoc(id, { title: `Run of ${plan}`, subject: null, type: 'test-run', stage: 'running', fields: { plan, build: null, results: [], summary: '' }, ...extra });

const planDoc = (id: string, cases: string[], extra: object = {}): Doc =>
  itemDoc(id, { title: `Plan number ${id.slice(2)}`, subject: null, type: 'test-plan', stage: 'ready', fields: { cases }, ...extra });

const runOn = (id: string, build: string | null, extra: object = {}): Doc =>
  testRunDoc(id, 'I-9', { fields: { plan: 'I-9', build, results: [], summary: '' }, ...extra });

const threadDoc = (id: string, extra: object = {}): Doc => ({ id, name: id, session: id, role: 'thread', status: 'active', ...extra });

const wakeDoc = (id: string, subject: string, extra: object = {}): Doc => ({
  id,
  subject,
  thread: 'lead',
  status: 'delivered',
  listeners: 1,
  at: NOW,
  ...extra,
});

const conflictEntry = (id: string, test: string, extra: object = {}): Doc => ({
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

const blankTest: TestDoc = {
  id: 'T-0',
  item: null,
  thread: null,
  title: '',
  covers: null,
  mode: 'manual',
  run: null,
  context: null,
  ac: [],
  status: 'pending',
  result: null,
  createdAt: EARLIER,
  updatedAt: EARLIER,
};

const blankRun: RunEntry = { id: 'R-0', name: '', status: 'exited', exitCode: 0, queuedAt: EARLIER, startedAt: EARLIER, endedAt: NOW, error: null, lastLine: null };

const asTestDoc = (doc: Doc): TestDoc => Object.assign({}, blankTest, doc);

const asRunEntry = (doc: Doc): RunEntry => Object.assign({}, blankRun, doc);

const byId = (docs: readonly object[], key = 'id'): Record<string, object> => Object.fromEntries(docs.map((doc) => [String(Reflect.get(doc, key)), doc]));

export type { Doc };
export { asRunEntry, asTestDoc, byId, conflictEntry, defDoc, doneDoc, itemDoc, launcherDoc, planDoc, runDoc, runOn, testDoc, testRunDoc, threadDoc, wakeDoc };
