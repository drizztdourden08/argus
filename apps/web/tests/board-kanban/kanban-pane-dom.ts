/* @layer renderer-app @kind test */
import { within } from '@testing-library/react';
import { expect } from 'vitest';
import type { Doc } from './kanban-harness';
import { item } from './kanban-harness';

const bug = (extra: Doc = {}): Doc =>
  item('I-1', {
    type: 'bug',
    stage: 'reported',
    title: 'Lobby',
    fields: { severity: 'S2', environment: 'bundle-16', repro: '- open the lobby\n- look', foundIn: 'T-57' },
    ...extra,
  });

const tabNames = (dialog: HTMLElement): string[] => within(dialog).getAllByRole('tab').map((tab) => tab.textContent);

const tabOf = (dialog: HTMLElement, name: string): HTMLElement => within(dialog).getByRole('tab', { name: new RegExp(`^${name}`) });

const labelled = (labels: string[]) => labels.map((label): string => expect.stringMatching(new RegExp(`^${label}`)) as string);

export { bug, labelled, tabNames, tabOf };
