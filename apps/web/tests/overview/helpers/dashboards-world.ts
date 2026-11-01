/* @layer renderer-app @kind test */
import { renderAt } from '../../helpers/render-at';
import { feed } from './feed';
import { minutesAgo } from './fixtures';
import type { ApiCall } from './stub-api';

const COUNTS = { threads: 2, working: 1, blocked: 1, needsYou: 2, itemsOpen: 5, tests: 10, testsPass: 7, testsFailing: 1, agents: 2 };

const MASTER = { name: 'master', title: 'Main checkout', cmd: 'game run', current: true, window: false, play: 'launcher-master', updatedAt: null, by: null, lastRun: null };

const cards = {
  rotp: {
    key: 'rotp', name: 'RotP', state: 'running', root: null, url: '/p/rotp/', counts: COUNTS, updatedAt: minutesAgo(1),
    attention: ['Q-3 needs a decision: ship it?', 'T-4 failing: login'],
    coordinator: { session: 'lead', link: null, lastSeen: minutesAgo(1), activity: 'working', note: null },
    master: MASTER,
  },
  beta: {
    key: 'beta', name: 'Beta', state: 'running', root: null, url: '/p/beta/', counts: { ...COUNTS, needsYou: 0, testsFailing: 0, blocked: 0, agents: 0, working: 0, itemsOpen: 1, tests: 0, testsPass: 0 }, master: null, updatedAt: minutesAgo(1),
    coordinator: { session: 'lead', link: null, lastSeen: minutesAgo(1), activity: 'idle', note: null },
  },
  delta: { key: 'delta', name: 'Delta', state: 'not started', root: null, url: '/p/delta/' },
};

const open = async (extra: Record<string, object> = {}) => {
  const view = renderAt('/');
  await feed(view.sources, { cards, ...extra });
  return view;
};

const posts = (calls: ApiCall[]): ApiCall[] => calls.filter((call) => call.method === 'POST');

export { cards, MASTER, open, posts };
