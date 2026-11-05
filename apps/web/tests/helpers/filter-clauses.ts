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

const pickView = async (trigger: RegExp, option: string): Promise<void> => {
  await openFilters();
  fireEvent.click(await screen.findByRole('button', { name: trigger }));
  await menuChoice(option);
};

const addGroup = async (option: string): Promise<void> => {
  await openFilters();
  fireEvent.click(await screen.findByRole('button', { name: 'Add group' }));
  await menuChoice(option);
};

const removeGroup = async (option: string): Promise<void> => {
  await openFilters();
  fireEvent.click(await screen.findByRole('button', { name: `Remove grouping by ${option}` }));
};

const chooseValue = async (fieldLabel: string, valueLabel: string): Promise<void> => {
  const trigger = await screen.findByRole('button', { name: `Change the ${fieldLabel} filter value` });
  if (trigger.getAttribute('aria-expanded') !== 'true') fireEvent.click(trigger);
  await menuChoice(valueLabel);
};

const addFilter = async (fieldLabel: string, valueLabel?: string): Promise<void> => {
  await openFilters();
  fireEvent.click(await screen.findByRole('button', { name: 'Add filter' }));
  await menuChoice(fieldLabel);
  if (valueLabel !== undefined) await chooseValue(fieldLabel, valueLabel);
};

const changeFilter = chooseValue;

const removeFilter = async (fieldLabel: string): Promise<void> => {
  fireEvent.click(await screen.findByRole('button', { name: `Remove filter on ${fieldLabel}` }));
};

export { addFilter, addGroup, changeFilter, openFilters, pickView, removeFilter, removeGroup };
