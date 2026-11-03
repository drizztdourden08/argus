/* @layer renderer-app @kind test */
import type { Doc } from './kanban-harness';
import { DAY, MIN, ago } from './kanban-harness';

const caseDoc = (id: string, extra: Doc = {}): Doc => ({
  id,
  item: 'I-1',
  thread: 'worker',
  title: `Case ${id}`,
  covers: 'gate PedestalScenes',
  build: 'b1',
  mode: 'manual',
  run: null,
  context: 'The game read the wrong level.',
  changed: ['kFeatures5 reads the hook'],
  setup: [{ label: 'profile', text: 'story-events' }],
  steps: ['open it', 'look at it'],
  pass: ['it opens'],
  fail: ['it breaks'],
  notes: ['Only the lead can tick I-1.'],
  ac: [1],
  status: 'pending',
  result: null,
  runs: [],
  proofs: [],
  rank: 1,
  rev: 1,
  createdAt: ago(DAY),
  updatedAt: ago(MIN),
  ...extra,
});

export { caseDoc };
