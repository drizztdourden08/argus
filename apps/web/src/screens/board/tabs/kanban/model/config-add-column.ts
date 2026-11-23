/* @layer renderer-app @kind logic */
import type { KanbanConfig } from '../../../../../data/types';
import { configUniqueId } from './config-unique-id';

const configAddColumn = (config: KanbanConfig): KanbanConfig => {
  const id = configUniqueId(config.columns.map((column) => column.id), 'column');
  return { ...config, columns: [...config.columns, { id, name: 'New column', match: { lanes: ['todo'], stages: [] }, rules: [] }] };
};

export { configAddColumn };
