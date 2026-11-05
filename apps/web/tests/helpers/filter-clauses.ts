/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';

const openFilters = async (): Promise<void> => {
  const toggle = await screen.findByRole('button', { name: /^Filters/ });
  if (toggle.getAttribute('aria-expanded') !== 'true') fireEvent.click(toggle);
  await screen.findByRole('search', { name: 'Filters' });
};

const menuChoice = async (name: string): Promise<void> => {
  const menus = await screen.findAllByRole('menu');
  const menu = menus.at(-1);
  if (!menu) throw new Error('no menu is open');
  fireEvent.click(await within(menu).findByText(name));
};

const valueButton = (field: string): HTMLElement | null => screen.queryByRole('button', { name: `Change the ${field} filter value` });

const pickClause = async (field: string, option: string): Promise<void> => {
  const chip = valueButton(field);
  if (chip) {
    fireEvent.click(chip);
  } else {
    fireEvent.click(await screen.findByRole('button', { name: 'Add filter' }));
    await menuChoice(field);
  }
  await menuChoice(option);
  const after = valueButton(field);
  if (after?.getAttribute('aria-expanded') === 'true') fireEvent.click(after);
};

const addFlag = async (field: string): Promise<void> => {
  fireEvent.click(await screen.findByRole('button', { name: 'Add filter' }));
  await menuChoice(field);
};

const pickView = async (trigger: RegExp, option: string): Promise<void> => {
  fireEvent.click(await screen.findByRole('button', { name: trigger }));
  await menuChoice(option);
};

export { addFlag, openFilters, pickClause, pickView };
