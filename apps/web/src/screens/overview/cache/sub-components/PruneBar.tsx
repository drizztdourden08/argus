/* @layer renderer-app @kind component */
import { Button, Inline, Stack, Text } from '@drizztdourden08/tessera';
import { NOTHING_TO_PRUNE } from '../cache.constants';
import type { PruneBarProps } from '../cache.type';
import { PruneConfirm } from './PruneConfirm';

const PruneBar = (props: PruneBarProps) => {
  const { report, flow } = props;
  const nothing = report.totals.prunableEntries === 0 && report.totals.abandonedFolders === 0;
  return (
    <Stack gap="sm">
      <Inline gap="sm" align="center" wrap>
        <Button size="sm" disabled={report.shadow || flow.busy || nothing || flow.asking} onClick={flow.ask}>Prune now</Button>
        {nothing && <Text variant="caption" tone="muted">{NOTHING_TO_PRUNE}</Text>}
      </Inline>
      {flow.asking && <PruneConfirm report={report} flow={flow} />}
    </Stack>
  );
};

export { PruneBar };
