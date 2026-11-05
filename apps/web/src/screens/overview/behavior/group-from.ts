/* @layer renderer-app @kind logic */
import type { GroupMode } from '../boards/boards-view.type';
import { GROUP_OPTIONS } from '../overview.constants';

const groupFrom = (id: string | null): GroupMode => (GROUP_OPTIONS.some((option) => option.id === id) ? (id as GroupMode) : 'none');

export { groupFrom };
