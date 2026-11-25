/* @layer renderer-app @kind component */
import { Tag, Text } from '@drizztdourden08/tessera';
import { ProjectLink } from '../../projects';
import { Age, FrameCode, RecordFrame, RecordGroup } from '../../../../shell';
import { GOOD_LOOK } from '../boards-run.constants';
import type { RunningNowProps } from '../boards.type';

const RunningNow = (props: RunningNowProps) => {
  const { runs } = props;
  if (runs.length === 0) return null;
  return (
    <RecordGroup header={<Text weight={600}>Running now</Text>} count={runs.length} label="Running now">
      {runs.map((run) => (
        <RecordFrame
          key={`${run.scope}/${run.id}`}
          name={run.id}
          density="row"
          icon={<Tag {...GOOD_LOOK}>{run.status}</Tag>}
          code={<FrameCode>{run.id}</FrameCode>}
          title={<Text title={run.name}>{run.name}</Text>}
          content={
            <>
              <ProjectLink project={run.scope} hash="#agents" />
              <Age value={run.startedAt ?? run.queuedAt} />
              {run.lastLine && <Text variant="caption" tone="muted" clamp={1}>{run.lastLine}</Text>}
            </>
          }
          attrs={{ role: 'listitem' }}
        />
      ))}
    </RecordGroup>
  );
};

export { RunningNow };
