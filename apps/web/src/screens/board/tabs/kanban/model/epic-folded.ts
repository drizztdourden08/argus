/* @layer renderer-app @kind logic */
import { epicKeyOf } from './epic-key';

const epicFolded = (stored: readonly string[], keys: readonly string[]): string[] => [
  ...stored.filter((key) => !epicKeyOf(key)),
  ...keys.filter((key) => !stored.includes(key)),
];

export { epicFolded };
