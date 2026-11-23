/* @layer renderer-app @kind component */
import { BrandMark, Flex, Text } from '@drizztdourden08/tessera';
import type { WorkingLineProps } from '../OverlordChat.type';
import '../../overlord-mark.css';

const WorkingLine = (props: WorkingLineProps) => {
  const { activity } = props;
  return (
    <Flex gap="sm" align="center">
      <BrandMark app="argus" size="sm" className={`overlord-mark is-${activity.mark}`} />
      <Text variant="caption" tone="muted" clamp={1}>{activity.text}</Text>
    </Flex>
  );
};

export { WorkingLine };
