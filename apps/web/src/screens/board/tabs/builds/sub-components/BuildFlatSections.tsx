/* @layer renderer-app @kind component */
import { Box, Flex, Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import type { BuildFlatSectionsProps } from '../builds-props.type';
import { groupByKind, isMaster } from '../model';
import { LauncherList } from './LauncherList';
import { MasterGroup } from './MasterGroup';

const BuildFlatSections = (props: BuildFlatSectionsProps) => {
  const { groups, group, views, boardKey, runs } = props;
  const rest = views.filter((view) => !isMaster(view.launcher));
  return (
    <Flex direction="column" gap="md">
      {groups.master.length > 0 && <MasterGroup views={groups.master} boardKey={boardKey} runs={runs} />}
      {group === 'kind'
        ? groupByKind(rest).map((entry) => (
          <Box key={entry.label} data-group={entry.label}>
            <RecordGroup header={<Text weight={600}>{entry.label}</Text>} count={entry.views.length} label={entry.label}>
              <LauncherList views={entry.views} label={entry.label} boardKey={boardKey} runs={runs} />
            </RecordGroup>
          </Box>
        ))
        : rest.length > 0 && <LauncherList views={rest} label="Builds" boardKey={boardKey} runs={runs} />}
    </Flex>
  );
};

export { BuildFlatSections };
