/* @layer renderer-app @kind component */
import { formatMemory } from '../../../../format';
import { procLabel } from '../model';
import type { KillRowConfirmProps } from '../processes.type';
import { KillConfirm } from './KillConfirm';

const KillRowConfirm = (props: KillRowConfirmProps) => {
  const { row, kill } = props;
  const proc = row.item;
  return (
    <KillConfirm
      question={`Kill ${procLabel(proc)} (${formatMemory(proc.memMB)})?`}
      busy={kill.killing}
      onKill={() => kill.killProcess(proc.id, false)}
      onKillTree={() => kill.killProcess(proc.id, true)}
      onCancel={kill.cancel}
    />
  );
};

export { KillRowConfirm };
