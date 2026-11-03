/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { expect } from 'vitest';

const pickItem = async (box: HTMLElement, hidden: readonly RegExp[]): Promise<HTMLButtonElement> => {
  const link = within(box).getByRole<HTMLButtonElement>('button', { name: 'Link' });
  expect(link.disabled).toBe(true);
  fireEvent.click(within(box).getByRole('combobox', { name: 'Item' }));
  hidden.forEach((name) => expect(screen.queryByRole('option', { name })).toBeNull());
  fireEvent.click(await screen.findByRole('option', { name: /^I-3/ }));
  return link;
};

export { pickItem };
