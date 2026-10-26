/* @layer renderer-app @kind component */
import { DateText, Flex, Text } from '@drizztdourden08/tessera';
import { summaryParts } from '../model';
import type { WorktreeSummaryProps } from '../worktrees-props.type';

const WorktreeSummary = (props: WorktreeSummaryProps) => {
  const { counts, meta } = props;
  if (!meta?.rules?.length) return null;
  const parts = summaryParts(counts, meta);
  return (
    <Flex gap="md" align="baseline" wrap>
      <Text variant="caption" tone="muted">{parts.counts}</Text>
      <Text variant="caption" tone="muted">{parts.need}</Text>
      <Text variant="caption" tone="muted">
        {meta.scannedAt ? <>scanned <DateText value={meta.scannedAt} format="elapsed" /></> : 'not scanned yet'}
      </Text>
      {parts.gone && <Text variant="caption" tone="muted">{parts.gone}</Text>}
      {parts.problems && <Text variant="caption" tone="warning">{parts.problems}</Text>}
      {parts.many && <Text variant="caption" tone="warning">{parts.many}</Text>}
    </Flex>
  );
};

export { WorktreeSummary };
