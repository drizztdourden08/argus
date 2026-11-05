/* @layer renderer-app @kind constants */
import type { LegendEntry } from './BoardMore.type';

const HIDDEN_KEY = 'coord.hidden';

const HIDDEN_TIP = 'Done threads, archived threads and unnamed new sessions';

const RESOURCES_NOTE = 'A thread claims a resource before it uses the shared thing and releases it after.';

const LEGEND_ENTRIES: readonly LegendEntry[] = [
  { label: 'working', look: { variant: 'urgency', color: 'success' }, text: 'An agent is running. It turns stale after 45 minutes without an update.' },
  { label: 'waiting', look: { variant: 'urgency', color: 'warning' }, text: 'An agent is waiting for something, often for you.' },
  { label: 'stopping', look: { variant: 'urgency', color: 'danger' }, text: 'You asked the lead to stop it; it is told at its next check.' },
  { label: 'pass', look: { variant: 'urgency', color: 'success' }, text: 'A test passed.' },
  { label: 'fail', look: { variant: 'urgency', color: 'danger' }, text: 'A test failed (partial and blocked are in the same family).' },
  { label: 'to run', look: { variant: 'urgency', color: 'info' }, text: 'A test still waiting for a result. One click on pass, fail, partial or blocked posts it.' },
  { label: 'needs you', look: { variant: 'urgency', color: 'danger' }, text: 'A question, a to-do or a test waits on you. The kinds below say which.' },
  { label: 'conflict', look: { variant: 'urgency', color: 'danger' }, text: 'A thread changed the same field after you looked. Nothing of yours was applied.' },
  { label: 'running', look: { variant: 'urgency', color: 'success' }, text: 'A launch the lead defined is running on this machine (queued, exited, failed and stopped are its other states).' },
  { label: 'bundle-2', look: { variant: 'normal', color: 'neutral' }, text: 'The build tag a test carries: a bundle. The one handed to you is filled and sits first.' },
  { label: 'current', look: { variant: 'urgency', color: 'success' }, text: 'On the Builds tab: the launcher that is valid to start now. A bundle shows the launcher it starts as a link to that tab.' },
  { label: 'ui notified', look: { variant: 'urgency', color: 'success' }, text: 'Your click woke the thread, which was listening.' },
  { label: 'ui not listening', look: { variant: 'normal', color: 'neutral' }, text: 'Your click was saved, but nobody was listening for that thread; it sees the result the next time it runs.' },
];

export { HIDDEN_KEY, HIDDEN_TIP, LEGEND_ENTRIES, RESOURCES_NOTE };
