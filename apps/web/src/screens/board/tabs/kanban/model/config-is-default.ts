/* @layer renderer-app @kind logic */
import type { KanbanConfig } from '../../../../../data/types';
import { DEFAULT_CONFIG } from './config-editor.constants';

const signature = (config: KanbanConfig): string =>
  JSON.stringify([
    config.columns.map((c) => [c.id, c.name, c.accent ?? null, c.match.lanes, c.match.stages ?? [], c.limit ?? null, c.rules ?? []]),
    (config.rules ?? []).map((r) => [r.id, r.kind, r.params?.ms ?? null, r.mode ?? null, r.alert?.to ?? null, r.alert?.wake ?? null, r.alert?.repeatMs ?? null]),
  ]);

const configIsDefault = (config: KanbanConfig): boolean => signature(config) === signature(DEFAULT_CONFIG);

export { configIsDefault };
