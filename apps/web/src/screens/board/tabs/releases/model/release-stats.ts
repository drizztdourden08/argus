/* @layer renderer-app @kind logic */
import type { ReleaseChannel } from '../../../../../data/remote';
import type { TabStat } from '../../../shared';

const releaseStats = (channels: readonly ReleaseChannel[]): TabStat[] => {
  const versions = channels.flatMap((channel) => channel.versions);
  const waiting = versions.filter((version) => version.replay?.state === 'pending').length;
  return [
    { id: 'channels', label: 'channels', count: channels.length, tone: 'neutral' },
    { id: 'versions', label: 'versions', count: versions.length, tone: 'neutral' },
    { id: 'waiting', label: 'wait for the go', count: waiting, tone: waiting > 0 ? 'warning' : 'neutral', tip: 'Not on GitHub yet: the replay runs on the owner\'s go' },
  ];
};

export { releaseStats };
