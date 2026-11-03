/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { expect, vi } from 'vitest';

const rowOf = (name: string): HTMLElement => {
  const found = document.body.querySelector<HTMLElement>(`[data-worktree="id-${name}"]`);
  if (!found) throw new Error(`no row ${name}`);
  return found;
};

const findRow = async (name: string): Promise<HTMLElement> => {
  await waitFor(() => expect(document.body.querySelector(`[data-worktree="id-${name}"]`)).not.toBeNull());
  return rowOf(name);
};

const text = (element: HTMLElement): string => element.textContent;

const openMenu = async (name: string): Promise<void> => {
  vi.stubGlobal('CSS', { supports: () => false });
  fireEvent.click(within(await findRow(name)).getByRole('button', { name: /^More actions for/ }));
  await screen.findByRole('menuitem', { name: /^Copy id/ });
};

const hasAction = (label: RegExp): boolean => screen.queryByRole('menuitem', { name: label }) !== null;

const chooseAction = async (name: string, label: RegExp): Promise<void> => {
  await openMenu(name);
  fireEvent.click(screen.getByRole('menuitem', { name: label }));
};

const messageBox = (): HTMLTextAreaElement => screen.getByRole<HTMLTextAreaElement>('textbox', { name: 'Message for the thread' });

const typeMessage = (value: string): void => {
  fireEvent.change(messageBox(), { target: { value } });
};

const confirmGroup = (label: string): HTMLElement => screen.getByRole('group', { name: label });

export { chooseAction, confirmGroup, findRow, hasAction, messageBox, openMenu, rowOf, text, typeMessage };
