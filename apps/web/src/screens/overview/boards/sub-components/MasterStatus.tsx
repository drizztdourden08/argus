/* @layer renderer-app @kind component */
import { Button, Text } from '@drizztdourden08/tessera';
import type { MasterStatusProps } from '../boards.type';

const MasterStatus = (props: MasterStatusProps) => {
  const { line, words, run, onLog } = props;
  return (
    <>
      <Text variant="caption" tone={line.tone === 'neutral' ? 'muted' : line.tone} role="status" aria-label={`Run ${run.id}`} title={`${run.lastLine ?? ''} (${run.id})`}>
        {words}
      </Text>
      <Button size="xs" variant="ghost" aria-label={`Log of ${run.id}`} onClick={() => onLog(run)}>Log</Button>
    </>
  );
};

export { MasterStatus };
