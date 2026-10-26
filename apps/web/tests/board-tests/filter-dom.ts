/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';
import { buildDoc, itemDoc, testDoc, threadDoc } from './fixtures';

const filterBoard = () => ({
  builds: [buildDoc('b1'), buildDoc('b2')],
  items: [itemDoc('I-1', { thread: 'lead', subject: 'work' }), itemDoc('I-2', { thread: 'coder', subject: 'play' })],
  threads: [threadDoc('lead'), threadDoc('coder')],
  tests: [
    testDoc('T-1', { title: 'Login works', item: 'I-1', build: 'b1', subject: 'work' }),
    testDoc('T-2', { title: 'Logout works', item: 'I-2', build: 'b2', subject: 'play', mode: 'auto' }),
    testDoc('T-3', { title: 'Sound plays', item: 'I-1', build: null, subject: 'work', status: 'fail' }),
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

export { filterBoard, pick, searchBox, stored, typeSearch };
