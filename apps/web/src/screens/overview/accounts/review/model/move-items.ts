/* @layer renderer-app @kind logic */
import type { MenuItem } from '@drizztdourden08/tessera';
import { NO_FOLDER, NO_FOLDER_VALUE } from '../review.constants';

const moveItems = (folders: readonly string[], onPick: (folder: string | null) => void): MenuItem[] => [
  { id: NO_FOLDER_VALUE, label: NO_FOLDER, icon: 'folder', onSelect: () => onPick(null) },
  ...folders.map((name): MenuItem => ({ id: `folder:${name}`, label: name, icon: 'folder', onSelect: () => onPick(name) })),
];

export { moveItems };
