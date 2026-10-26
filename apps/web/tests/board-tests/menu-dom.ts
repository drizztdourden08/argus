/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { expect } from 'vitest';
import type { Call } from './mount-tab';
import { resultCalls } from './outcome-dom';

const entries = (calls: Call[]): Call[] => calls.filter((call) => call.url.endsWith('/entries'));

const openMenu = async (id: string): Promise<void> => {
  fireEvent.click(await screen.findByRole('button', { name: `More actions for ${id}` }));
};

const choose = async (name: string): Promise<void> => {
  fireEvent.click(await screen.findByRole('menuitem', { name }));
};

const openFrom = async (id: string, item: string): Promise<HTMLElement> => {
  await openMenu(id);
  await choose(item);
  return screen.findByRole('dialog');
};

const firstResult = async (calls: Call[]): Promise<unknown> => {
  await waitFor(() => expect(resultCalls(calls)).toHaveLength(1));
  return resultCalls(calls)[0]?.body;
};

export { choose, firstResult, entries, openFrom, openMenu };
