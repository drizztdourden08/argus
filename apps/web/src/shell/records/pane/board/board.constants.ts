/* @layer renderer-app @kind constants */
const BOARD_KIND = 'board';

const BOARD_LABEL = 'Board';

const LANES: readonly { lane: string; label: string }[] = [
  { lane: 'doing', label: 'Doing' },
  { lane: 'review', label: 'Review' },
  { lane: 'todo', label: 'To do' },
];

const FAILING_STATUS = 'fail';

const FINISHED_AGENT_STATUSES: readonly string[] = ['done', 'closed', 'stopped', 'failed', 'lost', 'archived'];

const NO_IMAGES = { images: [] as number[] };

const EDIT_DONE: Readonly<Record<string, string>> = { goal: 'Saved the goal.', note: 'Saved the note.' };

const EDIT_EMPTY: Readonly<Record<string, string>> = { goal: 'add a goal', note: 'add a note' };

const RECENT_MAX = 12;

export { BOARD_KIND, BOARD_LABEL, EDIT_DONE, EDIT_EMPTY, FAILING_STATUS, FINISHED_AGENT_STATUSES, LANES, NO_IMAGES, RECENT_MAX };
