/* @layer renderer-app @kind component */
import { Inline, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { formatBytes } from '../../../../format';
import { Age } from '../../../../shell';
import { CALM_LOOK, GOOD_LOOK, WARN_LOOK } from '../cache.constants';
import type { CacheSummaryProps } from '../cache.type';

const CacheSummary = (props: CacheSummaryProps) => {
  const { report } = props;
  const { totals, lastPrune } = report;
  return (
    <Stack gap="xs">
      <Inline gap="sm" align="center" wrap role="status">
        <Tag {...CALM_LOOK} title="Cached package versions and Electron versions">{`${totals.versions} ${totals.versions === 1 ? 'version' : 'versions'}`}</Tag>
        <Tag {...CALM_LOOK} title="Everything in the cache folder">{`${formatBytes(totals.sizeBytes)} total`}</Tag>
        <Tag {...GOOD_LOOK} title="Used by a project or a worktree, never deleted">{`${formatBytes(totals.inUseBytes)} in use`}</Tag>
        <Tag {...(totals.unusedBytes > 0 ? WARN_LOOK : CALM_LOOK)} title="Used by no project and no worktree">{`${formatBytes(totals.unusedBytes)} unused`}</Tag>
        <Tag {...(totals.prunableBytes > 0 ? WARN_LOOK : CALM_LOOK)} title="Unused and past the grace period">{`${formatBytes(totals.prunableBytes)} ready to delete`}</Tag>
        <Text variant="caption">{lastPrune ? <>last prune <Age value={lastPrune.at} ago /></> : 'not pruned since Argus started'}</Text>
      </Inline>
      <Text variant="caption" tone="muted">{`${report.root}. An unused entry is deleted ${report.graceDays} days after its last use.`}</Text>
    </Stack>
  );
};

export { CacheSummary };
