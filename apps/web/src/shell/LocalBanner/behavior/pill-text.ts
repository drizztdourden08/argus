/* @layer renderer-app @kind logic */
import type { RemoteInfo } from '../../../data/remote';
import { PILL_PARTS } from '../LocalBanner.constants';

const pillText = (info: RemoteInfo): string => {
  const parts = PILL_PARTS.map((part) => {
    const count = info.replay[part.key] ?? 0;
    return `${count} ${count === 1 ? part.word : `${part.word}s`}`;
  });
  return ['Local', ...parts].join(' · ');
};

export { pillText };
