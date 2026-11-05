/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { BoardTextEditor } from '../../src/shell/records/pane/board/BoardTextEditor';
import { record } from '../overview-boards/helpers';

const post = vi.hoisted(() => vi.fn(() => Promise.resolve({ id: 'op-1', status: 'applied', outcome: null, entry: null, seq: 1 })));

vi.mock('../../src/data/boards/post-board-op', () => ({ postBoardOp: post }));

afterEach(() => {
  cleanup();
  post.mockClear();
});

describe('I-112 P5 the goal and note editor', { tags: ['critical'] }, () => {
  it('shows the stored goal and posts an edit of the goal when it is saved', async () => {
    const board = { ...record('argus', {}, { goal: 'Ship it', rev: 2 }), id: 'BD-1' };
    render(<BoardTextEditor record={board} field="goal" />);
    fireEvent.click(screen.getByRole('button', { name: 'Edit goal' }));
    const box = screen.getByRole('textbox');
    fireEvent.change(box, { target: { value: 'Ship it now' } });
    fireEvent.click(screen.getByRole('button', { name: /save/i }));
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    expect(post).toHaveBeenCalledWith({ kind: 'edit', id: 'BD-1', field: 'goal', value: 'Ship it now', prev: 'Ship it', baseRev: 2 });
  });

  it('posts the note field when the note is edited', async () => {
    const board = { ...record('argus'), id: 'BD-2' };
    render(<BoardTextEditor record={board} field="note" />);
    fireEvent.click(screen.getByRole('button', { name: 'Edit note' }));
    fireEvent.change(screen.getByRole('textbox'), { target: { value: 'Remember' } });
    fireEvent.click(screen.getByRole('button', { name: /save/i }));
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    expect(post).toHaveBeenCalledWith(expect.objectContaining({ field: 'note', value: 'Remember', prev: null }));
  });
});
