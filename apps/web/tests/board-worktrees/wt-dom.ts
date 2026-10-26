/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { expect } from 'vitest';

const rowOf = (name: string): HTMLElement => {
  const found = document.body.querySelector<HTMLElement>(`[data-marquee-key="id-${name}"]`);
  if (!found) throw new Error(`no row ${name}`);
  return found;
};

const rowNames = (): string[] =>
  [...document.body.querySelectorAll('[data-marquee-key]')].map((row) => (row.getAttribute('data-marquee-key') ?? '').replace(/^id-/, ''));

const rowButton = (name: string, label: string): HTMLButtonElement => within(rowOf(name)).getByRole<HTMLButtonElement>('button', { name: label });

const hasRowButton = (name: string, label: string): boolean => within(rowOf(name)).queryByRole('button', { name: label }) !== null;

const cellOf = (name: string, path: string): HTMLElement => {
  const found = rowOf(name).querySelector<HTMLElement>(`[data-column="${path}"]`);
  if (!found) throw new Error(`no cell ${path} in ${name}`);
  return found;
};

const text = (element: HTMLElement): string => element.textContent;

const findRow = async (name: string): Promise<HTMLElement> => {
  await waitFor(() => expect(document.body.querySelector(`[data-marquee-key="id-${name}"]`)).not.toBeNull());
  return rowOf(name);
};

const messageBox = (): HTMLTextAreaElement => screen.getByRole<HTMLTextAreaElement>('textbox', { name: 'Message for the thread' });

const typeMessage = (value: string): void => {
  fireEvent.change(messageBox(), { target: { value } });
};

const confirmGroup = (label: string): HTMLElement => screen.getByRole('group', { name: label });

const postedBodies = (calls: { url: string; body: unknown }[], suffix: string): unknown[] =>
  calls.filter((call) => call.url.endsWith(suffix)).map((call) => call.body);

export { cellOf, confirmGroup, findRow, hasRowButton, messageBox, postedBodies, rowButton, rowNames, rowOf, text, typeMessage };
