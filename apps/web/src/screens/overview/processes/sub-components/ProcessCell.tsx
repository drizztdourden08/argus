/* @layer renderer-app @kind component */
import { CopyValue, Tag, Text } from '@drizztdourden08/tessera';
import { clipText } from '../../format';
import { ProjectLink } from '../../projects';
import { STATUS_COLOR, statusTip } from '../model';
import { COMMAND_CLIP } from '../processes.constants';
import type { ProcessCellProps } from '../processes.type';
import { KillCell } from './KillCell';

const ProcessCell = (props: ProcessCellProps) => {
  const { row, path, kill } = props;
  const proc = row.item;
  const absent = <Text tone="muted">-</Text>;
  if (path === 'project') return proc.project ? <ProjectLink project={proc.project} /> : absent;
  if (path === 'status') {
    return proc.status ? <Tag variant="urgency" color={STATUS_COLOR[proc.status] ?? 'info'} title={statusTip(proc)}>{proc.status}</Tag> : absent;
  }
  if (path === 'cmd') return proc.cmd ? <CopyValue value={clipText(proc.cmd, COMMAND_CLIP)} text={proc.cmd} mono size="sm" label="Command line" /> : absent;
  return <KillCell proc={proc} kill={kill} />;
};

export { ProcessCell };
