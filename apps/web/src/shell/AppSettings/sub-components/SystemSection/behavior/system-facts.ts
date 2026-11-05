/* @layer renderer-app @kind logic */
import type { RemoteInfo } from '../../../../../data/remote';
import { formatAgo } from '../../../../../format';
import type { HealthInfo, SystemFact, SystemReads } from '../SystemSection.type';
import { UNKNOWN } from './system-facts.constants';

const modeOf = (health: HealthInfo | null): string => {
  if (!health) return UNKNOWN;
  if (health.shadow) return 'shadow';
  return health.preview ? 'preview' : 'live';
};

const versionOf = ({ health, version }: SystemReads): string => {
  const number = version?.version ?? health?.version;
  if (!number) return UNKNOWN;
  return version?.schema === undefined ? number : `${number} (schema ${version.schema})`;
};

const lastSync = (remote: RemoteInfo | null, now: number): string => {
  const syncs = Object.entries(remote?.syncs ?? {});
  if (syncs.length === 0) return 'never';
  const [key, sync] = syncs.reduce((a, b) => (Date.parse(b[1].at) > Date.parse(a[1].at) ? b : a));
  return `${key} ${formatAgo(sync.at, now)} ago`;
};

const systemFacts = (reads: SystemReads, now: number): SystemFact[] => [
  { label: 'Version', value: versionOf(reads) },
  { label: 'Started', value: reads.health?.startedAt ? `${formatAgo(reads.health.startedAt, now)} ago` : UNKNOWN },
  { label: 'Port', value: reads.health?.port === undefined ? UNKNOWN : String(reads.health.port) },
  { label: 'Mode', value: modeOf(reads.health) },
  { label: 'Remote', value: reads.remote?.mode ?? UNKNOWN },
  { label: 'Last sync', value: lastSync(reads.remote, now) },
];

export { systemFacts };
