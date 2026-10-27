/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { EXPLAINER_TEXT } from '../worktrees.constants';
import { summaryParts } from '../model';
import type { WorktreeSummaryProps } from '../worktrees-props.type';

const WorktreeSummary = (props: WorktreeSummaryProps) => {
  const { counts, meta } = props;
  if (!meta?.rules?.length) return <Text variant="caption" tone="dim">{EXPLAINER_TEXT}</Text>;
  const parts = summaryParts(counts, meta);
  const tail = [parts.gone, parts.problems, parts.many].filter(Boolean);
  return (
    <Flex as="span" direction="column" gap="2xs">
      <Text variant="caption" tone="muted">
        {`${parts.counts} · ${parts.need} · `}
        {meta.scannedAt ? <>scanned <Age value={meta.scannedAt} /> ago</> : 'not scanned yet'}
        {tail.map((part) => ` · ${part}`)}
      </Text>
      <Text variant="caption" tone="dim">{EXPLAINER_TEXT}</Text>
    </Flex>
  );
};

export { WorktreeSummary };
