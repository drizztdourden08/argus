/* @layer renderer-app @kind component */
import { Flex, Stack, Status, Text } from '@drizztdourden08/tessera';
import { ERROR_CLAMP_LINES, TOOL_STATUS } from '../OverlordChat.constants';
import type { ToolLineProps } from '../OverlordChat.type';

const ToolLine = (props: ToolLineProps) => {
  const { tool } = props;
  return (
    <Stack gap="2xs">
      <Flex gap="sm" align="center" className="overlord-chat__tool">
        <Text variant="caption" className="overlord-chat__tool-name">{tool.name}</Text>
        <Text variant="caption" tone="muted" className="overlord-chat__tool-summary">{tool.summary}</Text>
        <Status map={TOOL_STATUS} value={tool.status} size="sm" />
      </Flex>
      {tool.error && <Text variant="caption" tone="danger" clamp={ERROR_CLAMP_LINES}>{tool.error}</Text>}
    </Stack>
  );
};

export { ToolLine };
