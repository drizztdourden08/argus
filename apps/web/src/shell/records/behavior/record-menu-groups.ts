/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';
import { MENU_GROUP_IDS, RECORD_ACTIONS } from '../record-actions.constants';
import type { ActionDef, ActionHandlers } from '../records.type';

const itemOf = (def: ActionDef, handlers: ActionHandlers): MenuItem => ({ id: def.id, label: def.label, tone: def.tone, onSelect: () => handlers[def.id]?.() });

const groupOf = (group: ActionDef['group'], defs: readonly ActionDef[], handlers: ActionHandlers, label?: string): MenuGroup[] => {
  const items = defs.filter((def) => def.group === group && handlers[def.id]).map((def) => itemOf(def, handlers));
  return items.length > 0 ? [{ id: MENU_GROUP_IDS[group], label, items }] : [];
};

const recordMenuGroups = (kind: string, code: string, handlers: ActionHandlers, middle: readonly MenuGroup[] = []): MenuGroup[] => {
  const defs = RECORD_ACTIONS[kind] ?? [];
  return [...groupOf('open', defs, handlers, code), ...middle, ...groupOf('act', defs, handlers), ...groupOf('soft', defs, handlers)];
};

export { recordMenuGroups };
