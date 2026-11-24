/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

const setTitleMax = vi.fn<(key: string, value: number | null) => Promise<unknown>>();

vi.mock('../../src/data/posts', () => ({ setTitleMax: (key: string, value: number | null) => setTitleMax(key, value) }));
vi.mock('../../src/data/react', () => ({ useDoc: () => ({ doc: { project: 'argus', name: 'Argus', updatedAt: '', titleMax: null } }) }));

const { TitleLimitSection } = await import('../../src/screens/board/BoardSettings/sub-components/TitleLimitSection');

afterEach(() => {
  cleanup();
  setTitleMax.mockReset();
});

describe('the title length limit row of the board settings', { tags: ['critical'] }, () => {
  it('shows the default of 80 and sends the typed value on Save', async () => {
    setTitleMax.mockResolvedValue({ id: 'O-1', status: 'applied', outcome: null, entry: null, seq: 1 });
    render(<TitleLimitSection boardKey="argus" />);
    const input = screen.getByRole('spinbutton', { name: 'Title length limit' });
    expect(input).toHaveProperty('value', '80');
    fireEvent.change(input, { target: { value: '120' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(setTitleMax).toHaveBeenCalledWith('argus', 120));
  });

  it('keeps Save off until the value changes', () => {
    render(<TitleLimitSection boardKey="argus" />);
    expect(screen.getByRole('button', { name: 'Save' })).toHaveProperty('disabled', true);
  });
});
