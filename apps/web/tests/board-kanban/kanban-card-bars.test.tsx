/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { cleanKanban, item, mountKanban } from './kanban-harness';

cleanKanban();

const criteria = [
  { n: 1, text: 'The first thing works', done: true },
  { n: 2, text: 'The second thing works', done: false },
];

describe('I-95 the criteria bar of a card', { tags: ['critical'] }, () => {
  it('unrolls the criteria under the bar, folds them again and leaves the card closed', async () => {
    await mountKanban({ items: [item('I-1', { ac: criteria })] });
    expect(screen.queryByText('The first thing works')).toBeNull();
    fireEvent.click(await screen.findByRole('button', { name: 'Acceptance criteria, press to unroll' }));
    expect(screen.getByText('The first thing works')).toBeTruthy();
    expect(screen.getByText('The second thing works')).toBeTruthy();
    expect(screen.queryByRole('dialog', { name: 'Work item' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Acceptance criteria, press to fold' }));
    expect(screen.queryByText('The first thing works')).toBeNull();
  });
});
