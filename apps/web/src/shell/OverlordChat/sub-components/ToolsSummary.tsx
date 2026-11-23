/* @layer renderer-app @kind component */
import { Flex, Spinner, Tag, Text } from '@drizztdourden08/tessera';
import { countStatus } from '../behavior/count-status';
import { TOOL_COUNT_WORDS, TOOL_NAMES } from '../OverlordChat.constants';
import type { ToolsGroupProps } from '../OverlordChat.type';

const ToolsSummary = (props: ToolsGroupProps) => {
  const { tools } = props;
  const done = countStatus(tools, 'done');
  const failed = countStatus(tools, 'failed');
  const running = countStatus(tools, 'running');
  const name = tools.length === 1 ? TOOL_NAMES.one : TOOL_NAMES.many;
  return (
    <Flex gap="sm" align="center" wrap>
      <Text variant="caption" tone="muted">{tools.length} {name}</Text>
      {done > 0 && <Tag variant="urgency" color="success">{done} {TOOL_COUNT_WORDS.done}</Tag>}
      {failed > 0 && <Tag variant="urgency" color="danger">{failed} {TOOL_COUNT_WORDS.failed}</Tag>}
      {running > 0 && <Spinner size="sm" label={TOOL_COUNT_WORDS.running} />}
      {running > 0 && <Tag variant="urgency" color="info">{running} {TOOL_COUNT_WORDS.running}</Tag>}
    </Flex>
  );
};

export { ToolsSummary };
