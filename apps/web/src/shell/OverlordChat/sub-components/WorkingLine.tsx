/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { OverlordMark } from '../../OverlordMark';
import type { WorkingLineProps } from '../OverlordChat.type';

const WorkingLine = (props: WorkingLineProps) => {
  const { activity } = props;
  return (
    <Flex gap="sm" align="center">
      <OverlordMark activity={activity.mark} size="sm" />
      <Text variant="caption" tone="muted" clamp={1}>{activity.text}</Text>
    </Flex>
  );
};

export { WorkingLine };
