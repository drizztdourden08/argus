/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { expect } from 'vitest';
import { itemDoc, testDoc, threadDoc } from './fixtures';

const filterBoard = () => ({
  items: [itemDoc('I-1', { thread: 'lead', subject: 'work' }), itemDoc('I-2', { thread: 'coder', subject: 'play' })],
  threads: [threadDoc('lead'), threadDoc('coder')],
  tests: [
    testDoc('T-1', { title: 'Login works', item: 'I-1', subject: 'work' }),
    testDoc('T-2', { title: 'Logout works', item: 'I-2', subject: 'play', mode: 'auto' }),
    testDoc('T-3', { title: 'Sound plays', item: 'I-1', subject: 'work', status: 'fail' }),
  ],
});

const searchBox = (): HTMLInputElement => screen.getByRole<HTMLInputElement>('searchbox');

const typeSearch = (value: string): void => {
  fireEvent.change(searchBox(), { target: { value } });
};

const pick = async (select: string, option: string): Promise<void> => {
  const box = await screen.findByRole('combobox', { name: select });
  if (box.getAttribute('aria-expanded') !== 'true') fireEvent.click(box);
  fireEvent.click(await screen.findByRole('option', { name: option }));
};

const stored = (key: string): Record<string, unknown> => JSON.parse(localStorage.getItem(key) ?? '{}') as Record<string, unknown>;

const TITLES: Record<string, string> = { 'T-1': 'Login works', 'T-2': 'Logout works', 'T-3': 'Sound plays' };

const shown = (...ids: string[]): void => {
  for (const [id, title] of Object.entries(TITLES)) expect(screen.queryByText(title) !== null).toBe(ids.includes(id));
};

export { filterBoard, pick, shown, stored, typeSearch };
