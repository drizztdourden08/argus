/* @layer renderer-app @kind constants */
import type { ReplayCounts } from '../../data/remote';

const BANNER_PARTS: readonly { key: keyof ReplayCounts; word: string }[] = [
  { key: 'prs', word: 'pull request' },
  { key: 'releases', word: 'release' },
  { key: 'packages', word: 'package' },
  { key: 'checks', word: 'check run' },
];

const PILL_PARTS: readonly { key: keyof ReplayCounts; word: string }[] = [
  { key: 'prs', word: 'PR' },
  { key: 'releases', word: 'release' },
];

const BANNER_NONE = 'Local mode: nothing waits for the go';

export { BANNER_NONE, BANNER_PARTS, PILL_PARTS };
