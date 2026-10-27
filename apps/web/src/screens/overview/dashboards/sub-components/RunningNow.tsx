/* @layer renderer-app @kind component */
import { ListItemList, ListItemRow, Tag } from '@drizztdourden08/tessera';
import { ProjectLink } from '../../projects';
import { Age } from '../../../../shell';
import { GOOD_LOOK, RUNNING_HEADER } from '../dashboards.constants';
import type { RunningNowProps } from '../dashboards.type';

const RunningNow = (props: RunningNowProps) => {
  const { runs } = props;
  if (runs.length === 0) return null;
  return (
    <ListItemList label="Running now" heading="Running now" header={RUNNING_HEADER} count={runs.length}>
      {runs.map((run) => (
        <ListItemRow
          key={`${run.scope}/${run.id}`}
          icon={<Tag {...GOOD_LOOK}>{run.status}</Tag>}
          name={`${run.name} ${run.id}`}
          meta={run.lastLine ?? ''}
          columns={[
            { primary: <ProjectLink project={run.scope} hash="#agents" /> },
            { primary: <Age value={run.startedAt ?? run.queuedAt} />, align: 'end' },
          ]}
        />
      ))}
    </ListItemList>
  );
};

export { RunningNow };
