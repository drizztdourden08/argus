/* @layer renderer-app @kind component */
import { Button, Callout } from '@drizztdourden08/tessera';
import { formatMemory } from '../../format';
import { procLabel } from '../model';
import type { ProcessDetailProps } from '../processes.type';
import { KillConfirm } from './KillConfirm';

const ProcessKill = (props: ProcessDetailProps) => {
  const { proc, kill } = props;
  if (proc.protected) {
    const why = proc.protectedWhy ? ` (${proc.protectedWhy})` : '';
    return <Callout tone="info">{`Protected${why}: Argus never kills this process.`}</Callout>;
  }
  if (!proc.killable) return <Callout tone="info">The rule of this process does not allow killing it.</Callout>;
  if (kill.ask?.kind === 'proc' && kill.ask.id === proc.id) {
    return (
      <KillConfirm
        question={`Kill ${procLabel(proc)} (${formatMemory(proc.memMB)})?`}
        busy={kill.killing}
        onKill={() => kill.killProcess(proc.id, false)}
        onKillTree={() => kill.killProcess(proc.id, true)}
        onCancel={kill.cancel}
      />
    );
  }
  return (
    <Button variant="danger" size="sm" disabled={kill.killing} onClick={() => kill.askProcess(proc.id)} title={`Kill ${procLabel(proc)}`}>
      Kill
    </Button>
  );
};

export { ProcessKill };
