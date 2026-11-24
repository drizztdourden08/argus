/* @layer renderer-app @kind component */
import { BrandMark, Flex, Text } from '@drizztdourden08/tessera';
import type { WorkingLineProps } from '../OverlordChat.type';

const WorkingLine = (props: WorkingLineProps) => {
  const { activity } = props;
  return (
    <Flex gap="sm" align="center">
      <BrandMark app="argus" size="sm" activity={activity.mark} />
      <Text variant="caption" tone="muted" clamp={1} dots>{activity.text}</Text>
    </Flex>
  );
};

export { WorkingLine };
