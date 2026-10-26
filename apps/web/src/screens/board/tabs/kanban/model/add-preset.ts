/* @layer renderer-app @kind logic */
import { GENERAL } from './model.constants';
import type { AddPreset, ItemGroup } from './model.type';

const addPreset = (group: ItemGroup, column: string | null, focus: string): AddPreset => {
  if (column === null) return focus ? { thread: focus } : {};
  if (group === 'status') return { status: column, ...(focus ? { thread: focus } : {}) };
  if (group === 'thread') return { thread: column };
  return { ...(column === GENERAL ? {} : { subject: column }), ...(focus ? { thread: focus } : {}) };
};

export { addPreset };
