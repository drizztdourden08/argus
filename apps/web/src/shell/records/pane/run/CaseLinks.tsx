/* @layer renderer-app @kind component */
import { CopyValue, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { ShownCode } from '../../sub-components/ShownCode';
import type { CaseLinksProps } from './run.type';

const CaseLinks = (props: CaseLinksProps) => {
  const { test, open } = props;
  const { item, covers, run } = test;
  if (!item && !covers && !run) return null;
  return (
    <Flex direction="column" gap="xs">
      <Text variant="label">Links</Text>
      <Flex gap="xs" align="center" wrap>
        {item && <Tag color="neutral" title={`Open ${item}`} onClick={() => open(item)}><ShownCode id={item} /></Tag>}
        {covers && <Text variant="caption" tone="muted">{covers}</Text>}
      </Flex>
      {run && <CopyValue value={run} label="Run command" mono />}
    </Flex>
  );
};

export { CaseLinks };
