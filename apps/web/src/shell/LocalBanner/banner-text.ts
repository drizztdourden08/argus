/* @layer renderer-app @kind logic */
import type { RemoteInfo } from '../../data/remote';
import { BANNER_NONE, BANNER_PARTS } from './LocalBanner.constants';

const bannerText = (info: RemoteInfo): string | null => {
  if (info.mode !== 'local') return null;
  const counts = BANNER_PARTS.map((part) => ({ count: info.replay[part.key] ?? 0, word: part.word })).filter((part) => part.count > 0);
  if (counts.length === 0) return BANNER_NONE;
  const total = counts.reduce((sum, part) => sum + part.count, 0);
  const parts = counts.map((part) => `${part.count} ${part.count === 1 ? part.word : `${part.word}s`}`);
  return `Local mode: ${parts.join(', ')} ${total === 1 ? 'waits' : 'wait'} for the go`;
};

export { bannerText };
