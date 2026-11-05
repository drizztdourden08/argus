/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { ReleaseChannel } from '../../../../../data/remote';
import { toMillis } from '../../../../../format';
import type { ReleasesFilters, ReleasesOrder } from '../releases.type';

const newestStamp = (channel: ReleaseChannel): number =>
  channel.versions.reduce((latest, version) => {
    const at = toMillis(version.publishedAt);
    return Number.isFinite(at) && at > latest ? at : latest;
  }, 0);

const matchesQuery = (channel: ReleaseChannel, query: string): boolean => {
  const needle = query.trim().toLowerCase();
  if (needle === '') return true;
  return channel.channel.toLowerCase().includes(needle)
    || channel.versions.some((version) => version.version.toLowerCase().includes(needle) || version.tag.toLowerCase().includes(needle));
};

const compare = (order: ReleasesOrder, left: ReleaseChannel, right: ReleaseChannel): number =>
  order === 'channel' ? left.channel.localeCompare(right.channel) : newestStamp(left) - newestStamp(right);

const viewChannels = (channels: readonly ReleaseChannel[], filters: ReleasesFilters, order: ReleasesOrder, direction: SortDirection): ReleaseChannel[] => {
  const sign = direction === 'asc' ? 1 : -1;
  return channels
    .filter((channel) => (filters.channel === '' || channel.channel === filters.channel) && matchesQuery(channel, filters.query))
    .sort((left, right) => sign * compare(order, left, right));
};

export { viewChannels };
