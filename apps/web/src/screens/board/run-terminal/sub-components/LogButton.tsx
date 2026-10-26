/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import type { LogButtonProps } from '../run-terminal.type';

const LogButton = (props: LogButtonProps) => {
  const { run, onLog } = props;
  if (!run) return null;
  return <Button size="sm" variant="secondary" aria-label={`Log of ${run.id}`} onClick={() => onLog(run.id)}>Log</Button>;
};

export { LogButton };
