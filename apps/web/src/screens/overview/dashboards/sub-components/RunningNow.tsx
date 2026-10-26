/* @layer renderer-app @kind component */
import { DateText, ListItemList, ListItemRow, Tag } from '@drizztdourden08/tessera';
import { ProjectLink } from '../../projects';
import { GOOD_LOOK } from '../dashboards.constants';
import type { RunningNowProps } from '../dashboards.type';

const RunningNow = (props: RunningNowProps) => {
  const { runs } = props;
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
            { primary: <DateText value={run.startedAt ?? run.queuedAt} format="elapsed" />, align: 'end' },
          ]}
        />
      ))}
    </ListItemList>
  );
};

export { RunningNow };
