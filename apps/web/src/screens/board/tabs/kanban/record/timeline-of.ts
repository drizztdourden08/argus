/* @layer renderer-app @kind logic */
import { TIMELINE_KINDS } from './record.constants';
import type { TimelineInput, TimelineRow } from './record.type';

const entryRows = (input: TimelineInput): TimelineRow[] =>
  input.entries
    .filter((entry) => TIMELINE_KINDS.includes(entry.kind))
    .map((entry) => ({
      key: entry.id,
      at: entry.createdAt,
      kind: entry.kind,
      who: entry.author === 'human' ? 'you' : entry.author,
      id: entry.id,
      text: entry.text,
      status: entry.kind === 'question' && entry.status === 'open' && !input.opened.has(entry.id) ? 'answered' : (entry.status ?? null),
      on: entry.replyTo ?? null,
      doneable: false,
    }));

const stepRows = (input: TimelineInput): TimelineRow[] =>
  input.todos.map(({ action, done }) => ({
    key: action.id,
    at: action.createdAt ?? '',
    kind: 'step',
    who: action.thread ?? '',
    id: action.id,
    text: action.text ?? '',
    status: done ? 'done' : 'pending',
    on: null,
    doneable: !done,
  }));

const logRows = (input: TimelineInput): TimelineRow[] =>
  input.log.flatMap((doc) =>
    doc.entries
      .filter((line) => line.item === input.itemId)
      .map((line) => ({ key: `log-${doc.subject}-${line.seq}`, at: line.ts, kind: 'log', who: line.thread, id: '', text: line.text, status: null, on: null, doneable: false })));

const timelineOf = (input: TimelineInput): TimelineRow[] =>
  [...entryRows(input), ...stepRows(input), ...logRows(input)].sort((a, b) => b.at.localeCompare(a.at));

export { timelineOf };
