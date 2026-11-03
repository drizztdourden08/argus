/* @layer renderer-app @kind logic */
import { NO_PARENT, OWNER } from './model.constants';
import type { AddPreset, ItemGroup } from './model.type';

const withFocus = (preset: AddPreset, focus: string): AddPreset => (focus && !preset.thread ? { ...preset, thread: focus } : preset);

const groupPreset = (group: ItemGroup, column: string): AddPreset => {
  if (group === 'status') return { status: column };
  if (group === 'thread') return { thread: column };
  if (group === 'parent') return column === NO_PARENT ? {} : { parent: column };
  return column === OWNER ? { assignee: OWNER } : { thread: column };
};

const addPreset = (group: ItemGroup, column: string | null, focus: string): AddPreset =>
  withFocus(column === null ? {} : groupPreset(group, column), focus);

export { addPreset };
