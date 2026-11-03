/* @layer renderer-app @kind logic */
import { askKind } from './ask-kind';
import { isUnapplied } from './is-unapplied';
import { HUMAN } from './model.constants';
import { settledIds } from './settled-ids';
import type { EntryDocs, Needs, NeedsInput } from './model.type';

const pendingDone = (entries: EntryDocs): Set<string> =>
  new Set(entries.filter((entry) => entry.author === HUMAN && entry.kind === 'done' && entry.replyTo && isUnapplied(entry)).map((entry) => String(entry.replyTo)));

const bump = (map: Map<string, Needs>, item: string | null | undefined, field: keyof Needs): void => {
  if (!item) return;
  const needs = map.get(item) ?? { questions: 0, decisions: 0, todos: 0, tests: 0 };
  needs[field] += 1;
  map.set(item, needs);
};

const countEntries = (map: Map<string, Needs>, entries: EntryDocs): void => {
  const settled = settledIds(entries);
  for (const entry of entries) {
    const kind = askKind(entry, settled);
    if (kind) bump(map, entry.item, kind === 'question' ? 'questions' : 'decisions');
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
