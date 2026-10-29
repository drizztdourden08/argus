/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { byId, decision, freeText } from '../helpers/board-docs';
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

describe('a decision card with images', () => {
  it('shows the whole-entry image and each option image under its option', async () => {
    mockPosts();
    await openBoard({ entries: byId(pictured) });
    expect(screen.getByAltText('The current look').getAttribute('src')).toBe('/files/rotp/proofs/Q-1-1.png');
    const first = screen.getByText('Option 1: Yes, private').parentElement as HTMLElement;
    expect(within(first).getByAltText('Private repo page')).toBeTruthy();
    expect(within(first).queryByAltText('Local only')).toBeNull();
    const second = screen.getByText('Option 2: Not yet, stay local').parentElement as HTMLElement;
    expect(within(second).getByAltText('Local only')).toBeTruthy();
  });

  it('opens a thumbnail large in the lightbox and closes it', async () => {
    mockPosts();
    await openBoard({ entries: byId(pictured) });
    expect(screen.queryByRole('dialog', { name: 'Private repo page' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Open Private repo page' }));
    const viewer = screen.getByRole('dialog', { name: 'Private repo page' });
    fireEvent.click(within(viewer).getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Private repo page' })).toBeNull());
  });

  it('a question without images shows no thumbnails and no option titles', async () => {
    mockPosts();
    await openBoard({ entries: byId(decision, freeText) });
    expect(screen.queryByRole('group', { name: 'Screenshots' })).toBeNull();
    expect(screen.queryByText(/^Option \d/)).toBeNull();
  });
});
