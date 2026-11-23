/* @layer renderer-app @kind component */
import { Disclosure, Stack } from '@drizztdourden08/tessera';
import type { ToolsGroupProps } from '../OverlordChat.type';
import { ToolLine } from './ToolLine';
import { ToolsSummary } from './ToolsSummary';

const ToolsGroup = (props: ToolsGroupProps) => {
  const { tools } = props;
  return (
    <Disclosure size="sm" summary={<ToolsSummary tools={tools} />}>
      <Stack gap="2xs">
        {tools.map((tool) => <ToolLine key={tool.id} tool={tool} />)}
      </Stack>
    </Disclosure>
  );
};

export { ToolsGroup };
