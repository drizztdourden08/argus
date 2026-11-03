/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import { MASTER_NOTE, MASTER_TITLE } from '../builds.constants';
import type { MasterGroupProps } from '../builds-props.type';
import { GroupTitle } from './GroupTitle';
import { LauncherList } from './LauncherList';

const MasterGroup = (props: MasterGroupProps) => {
  const { views, boardKey, runs } = props;
  return (
    <Box data-group="master">
      <RecordGroup header={<GroupTitle title={MASTER_TITLE} note={MASTER_NOTE} />} count={views.length} label={MASTER_TITLE}>
        <LauncherList views={views} label={MASTER_TITLE} boardKey={boardKey} runs={runs} />
      </RecordGroup>
    </Box>
  );
};

export { MasterGroup };
