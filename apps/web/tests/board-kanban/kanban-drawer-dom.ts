/* @layer renderer-app @kind test */
import { waitFor } from '@testing-library/react';
import { expect } from 'vitest';
import type { Doc, Posted } from './kanban-harness';
import { item, mountKanban, openItem } from './kanban-harness';

const rich = (extra: Doc = {}): Doc =>
  item('I-1', {
    title: 'Login',
    description: 'Make **login** work. See [the spec](https://example.com/spec).\n\n- one\n- two',
    ac: [
      { n: 1, text: 'Shows an error', done: false },
      { n: 2, text: 'Keeps the session', done: true },
    ],
    ...extra,
  });

const tests = [
  { id: 'T-1', item: 'I-1', title: 'Wrong password', mode: 'manual', status: 'pass', ac: [1] },
  { id: 'T-2', item: 'I-1', title: 'Right password', mode: 'auto', status: 'pending', ac: [] },
  { id: 'T-3', item: 'I-9', title: 'Elsewhere', mode: 'auto', status: 'pass', ac: [] },
];

const opened = async (extra: Doc = {}) => {
  const mounted = await mountKanban({ items: [rich(extra)], tests });
  const dialog = await openItem('I-1');
  return { ...mounted, dialog };
};

const postedEdit = async (posted: Posted[], set: Doc, prev: Doc): Promise<void> => {
  await waitFor(() => expect(posted).toHaveLength(1));
  expect(posted[0]?.body).toMatchObject({ kind: 'edit', target: { c: 'items', id: 'I-1' }, set, prev, baseRev: 3 });
};

export { opened, postedEdit, rich };
