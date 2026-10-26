/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { act, cleanup, screen, waitFor } from '@testing-library/react';
import { expect, vi } from 'vitest';
import { renderBody } from './render-body';
import { seedBoard } from './seed-board';

const openBoard = async (docs: Record<string, Record<string, object>>) => {
  const view = renderBody();
  await waitFor(() => expect(view.sources).toHaveLength(1));
  act(() => seedBoard(view.sources[0], docs));
  return view;
};

const resetBoard = (): void => {
  cleanup();
  toast.clear();
  vi.unstubAllGlobals();
};

const textbox = (name: string) => screen.getByRole<HTMLTextAreaElement>('textbox', { name });

export { openBoard, resetBoard, textbox };
