/* @layer renderer-app @kind constants */

const AGENT_ANCHOR = /^A-\d+$/;

const AGENTS_HASH = 'agents';

const FINISHED_KEY = 'coord.agfin';

const FINISHED_HINT = 'Finished agents and runs stay as history; archived agents are in Trash.';

const NO_NOTE_LINE = 'no note';

const COLUMNS: readonly { id: string; label: string }[] = [
  { id: 'status', label: 'Status' },
  { id: 'agent', label: 'Agent' },
  { id: 'model', label: 'Model' },
  { id: 'kind', label: 'Kind' },
  { id: 'item', label: 'Item' },
  { id: 'branch', label: 'Branch' },
  { id: 'time', label: 'Time' },
  { id: 'now', label: 'Now' },
  { id: 'message', label: 'Last message' },
  { id: 'note', label: 'Note' },
  { id: 'actions', label: 'Actions' },
];


export { AGENT_ANCHOR, AGENTS_HASH, COLUMNS, FINISHED_HINT, FINISHED_KEY, NO_NOTE_LINE };
