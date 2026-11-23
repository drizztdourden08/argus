/* @layer renderer-app @kind logic */
import { EPIC_KEY_PREFIX, NO_EPIC_ID } from './epic.constants';
import type { EpicKey } from './model.type';

const epicKeyOf = (key: string): EpicKey | null => {
  if (!key.startsWith(EPIC_KEY_PREFIX)) return null;
  const rest = key.slice(EPIC_KEY_PREFIX.length);
  const cut = rest.indexOf(':');
  if (cut < 0) return null;
  const epic = rest.slice(0, cut);
  return { epic: epic === NO_EPIC_ID ? null : epic, column: rest.slice(cut + 1) };
};

export { epicKeyOf };
