/* @layer renderer-app @kind component */
import { Flex, ProgressBar, Text } from '@drizztdourden08/tessera';
import { progressText } from '../behavior/progress-text';
import type { BoardProgressProps } from '../boards.type';

const BoardProgress = (props: BoardProgressProps) => {
  const { progress } = props;
  if (progress.total === 0) return null;
  const text = progressText(progress);
  return (
    <Flex gap="xs" align="center" wrap>
      <ProgressBar value={progress.done} max={progress.total} tone="success" className="board-progress__bar" label={text} />
      <Text variant="caption" tone="muted">{text}</Text>
    </Flex>
  );
};

export { BoardProgress };
