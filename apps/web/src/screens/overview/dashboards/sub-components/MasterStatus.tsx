/* @layer renderer-app @kind component */
import { Badge, Button, Inline, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { RUN_DOT } from '../dashboards.constants';
import type { MasterStatusProps } from '../dashboards.type';
import './MasterStatus.css';

const MasterStatus = (props: MasterStatusProps) => {
  const { line, run, onLog } = props;
  return (
    <>
      <Inline gap="xs" align="center" className="master-status" role="status" aria-label={`Run ${run.id}`} title={`${run.lastLine ?? ''} (${run.id})`}>
        <Badge variant="dot" color={RUN_DOT[line.tone]} />
        <Text variant="caption" tone={line.tone === 'neutral' ? 'muted' : 'dim'} className="master-status__text">
          {line.head}
          {line.at ? <>{line.running ? ' ' : ' · '}<Age value={line.at} ago={!line.running} /></> : null}
          {line.tail}
        </Text>
      </Inline>
      <Button size="xs" variant="ghost" aria-label={`Log of ${run.id}`} onClick={() => onLog(run)}>Log</Button>
    </>
  );
};

export { MasterStatus };
