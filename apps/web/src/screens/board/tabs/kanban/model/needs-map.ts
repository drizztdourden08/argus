/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { isUnapplied } from './is-unapplied';
import { ANSWER_KINDS, HUMAN } from './model.constants';
import type { EntryDocs, Needs, NeedsInput } from './model.type';

const repliedTo = (entries: EntryDocs, kinds: readonly string[]): Set<string> =>
  new Set(entries.filter((entry) => entry.author === HUMAN && kinds.includes(entry.kind) && entry.replyTo).map((entry) => String(entry.replyTo)));

const pendingDone = (entries: EntryDocs): Set<string> =>
  new Set(entries.filter((entry) => entry.author === HUMAN && entry.kind === 'done' && entry.replyTo && isUnapplied(entry)).map((entry) => String(entry.replyTo)));

const bump = (map: Map<string, Needs>, item: string | null | undefined, field: keyof Needs): void => {
  if (!item) return;
  const needs = map.get(item) ?? { questions: 0, decisions: 0, todos: 0, tests: 0 };
  needs[field] += 1;
  map.set(item, needs);
};

const isOpenQuestion = (entry: WithId<Entry>, settled: ReadonlySet<string>): boolean =>
  entry.kind === 'question' && entry.status === 'open' && !settled.has(entry.id);

const countEntries = (map: Map<string, Needs>, entries: EntryDocs): void => {
  const settled = new Set([...repliedTo(entries, ANSWER_KINDS), ...repliedTo(entries, ['ack'])]);
  for (const entry of entries) {
    if (isOpenQuestion(entry, settled)) bump(map, entry.item, 'questions');
    else if (entry.kind === 'decision' && entry.status === 'pending') bump(map, entry.item, 'decisions');
  }
};

const countActions = (map: Map<string, Needs>, input: NeedsInput): void => {
  const done = pendingDone(input.entries);
  for (const action of input.actions) if (action.status === 'pending' && !done.has(action.id)) bump(map, action.item, 'todos');
};

const countTests = (map: Map<string, Needs>, tests: NeedsInput['tests']): void => {
  for (const test of tests) if (test.mode === 'manual' && test.status === 'pending' && !test.archivedAt) bump(map, test.item, 'tests');
};

const needsMap = (input: NeedsInput): Map<string, Needs> => {
  const map = new Map<string, Needs>();
  countEntries(map, input.entries);
  countActions(map, input);
  countTests(map, input.tests);
  return map;
};

export { needsMap };
