/* @layer renderer-app @kind component */
import { ListItemList, ListItemRow, Tag, Text } from '@drizztdourden08/tessera';
import { SECOND_MS, useNow } from '../../clock';
import { formatElapsed } from '../../format';
import { ProjectLink } from '../../projects';
import { GOOD_LOOK } from '../dashboards.constants';
import type { RunningNowProps } from '../dashboards.type';

const RunningNow = (props: RunningNowProps) => {
  const { runs } = props;
  const now = useNow(SECOND_MS);
  if (runs.length === 0) return null;
  return (
    <ListItemList label="Running now" heading="Running now" count={runs.length}>
      {runs.map((run) => (
        <ListItemRow
          key={`${run.scope}/${run.id}`}
          icon={<Tag {...GOOD_LOOK}>{run.status}</Tag>}
          name={`${run.name} ${run.id}`}
          meta={run.lastLine ?? ''}
          columns={[
            { primary: <ProjectLink project={run.scope} hash="#agents" /> },
            { primary: <Text numeric>{formatElapsed(now - Date.parse(run.startedAt ?? run.queuedAt))}</Text>, align: 'end' },
          ]}
        />
      ))}
    </ListItemList>
  );
};

export { RunningNow };
