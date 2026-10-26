/* @layer renderer-app @kind logic */
import type { RegistryItem } from './model.type';

const registryName = (row: RegistryItem, nameOf: (key: string) => string): string => {
  if (row.role === 'coordinator') return nameOf(row.project);
  return `${nameOf(row.project)} / ${row.session ?? row.thread ?? row.id}`;
};

export { registryName };
