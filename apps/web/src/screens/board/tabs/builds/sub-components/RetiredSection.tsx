/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import type { RetiredSectionProps } from '../builds-props.type';
import { LauncherList } from './LauncherList';

const RetiredSection = (props: RetiredSectionProps) => {
  const { views, boardKey, runs, compact } = props;
  if (views.length === 0) return null;
  return (
    <Box data-group="closed">
      <RecordGroup header={<Text weight={600}>Closed</Text>} count={views.length} label="Closed builds">
        <LauncherList views={views} label="Closed builds" boardKey={boardKey} runs={runs} compact={compact} />
      </RecordGroup>
    </Box>
  );
};

export { RetiredSection };
