/* @layer renderer-app @kind component */
import { Status, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { runLine } from '../../../../data/runs';
import type { RunStatusProps } from '../run-play.type';

const RunStatus = (props: RunStatusProps) => {
  const { run } = props;
  const line = runLine(run);
  const running = run.status === 'running';
  return (
    <Status tone={line.tone} dot pulse={running} role="status" aria-label={`Run ${run.id}`} title={`${line.last} (${run.id})`}>
      <Text variant="caption" tone="dim">
        {line.label}
        {line.since && <>{running ? ' ' : ' · '}<Age value={line.since} />{running ? '' : ' ago'}</>}
        {line.last && ` · ${line.last}`}
      </Text>
    </Status>
  );
};

export { RunStatus };
