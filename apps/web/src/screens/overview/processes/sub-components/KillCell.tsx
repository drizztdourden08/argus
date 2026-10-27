/* @layer renderer-app @kind component */
import { Button, Icon, Text } from '@drizztdourden08/tessera';
import { procLabel } from '../model';
import type { KillCellProps } from '../processes.type';

const KillCell = (props: KillCellProps) => {
  const { proc, kill } = props;
  if (proc.protected) {
    const why = proc.protectedWhy ? `Protected (${proc.protectedWhy}): Argus never kills this process` : 'Protected: Argus never kills this process';
    return <Text tone="muted" title={why} aria-label="Protected"><Icon name="lock" size={14} /></Text>;
  }
  if (!proc.killable) return <Text tone="muted" title="The rule of this process does not allow killing it">-</Text>;
  if (kill.ask?.kind === 'proc' && kill.ask.id === proc.id) return null;
  return (
    <Button size="xs" variant="danger" disabled={kill.killing} aria-label={`Kill ${procLabel(proc)}`} onClick={() => kill.askProcess(proc.id)}>
      Kill
    </Button>
  );
};

export { KillCell };
