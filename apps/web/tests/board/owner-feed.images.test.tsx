/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { byId, decision } from '../helpers/board-docs';
import { openBoard, resetBoard } from '../helpers/open-board';
import { mockPosts } from '../helpers/post-mock';

afterEach(resetBoard);

const pictured = {
  ...decision,
  images: [
    { n: 1, url: '/files/rotp/proofs/Q-1-1.png', caption: 'The current look' },
    { n: 2, url: '/files/rotp/proofs/Q-1-2.png', caption: 'Private repo page', option: 1 },
    { n: 3, url: '/files/rotp/proofs/Q-1-3.png', caption: 'Local only', option: 2 },
  ],
};

describe('S1 a decision card with images', { tags: ['standard'] }, () => {
  it('opens a thumbnail large in the lightbox and closes it', async () => {
    mockPosts();
    await openBoard({ entries: byId(pictured) });
    expect(screen.queryByRole('dialog', { name: 'Private repo page' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Open Private repo page' }));
    const viewer = screen.getByRole('dialog', { name: 'Private repo page' });
    fireEvent.click(within(viewer).getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Private repo page' })).toBeNull());
  });
});
