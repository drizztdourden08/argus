/* @layer renderer-app @kind logic */
import { epicKeyOf } from './epic-key';

const epicStored = (stored: readonly string[], keys: readonly string[], folded: readonly string[]): string[] => [
  ...stored.filter((key) => !epicKeyOf(key)),
  ...keys.filter((key) => !folded.includes(key)),
];

export { epicStored };
