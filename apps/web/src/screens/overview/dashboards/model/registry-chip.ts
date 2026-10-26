/* @layer renderer-app @kind logic */
import { registryLook } from './registry-look';
import { registryName } from './registry-name';
import type { RegistryChip, RegistryItem } from './model.type';

const registryChip = (row: RegistryItem, nameOf: (key: string) => string, now: number): RegistryChip => {
  const name = registryName(row, nameOf);
  const look = registryLook(row, name, now);
  const closed = row.state !== 'fault' && row.boardRequired === true && row.boardOpen !== true;
  return {
    name,
    ...look,
    tip: closed ? `${look.tip}. Its board tab is not open: the Overlord is told when this lasts.` : look.tip,
    extra: closed ? 'board closed' : '',
  };
};

export { registryChip };
