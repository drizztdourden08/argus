/* @layer renderer-app @kind component */
import { CopyValue, Text } from '@drizztdourden08/tessera';
import { clipText, formatMinutes } from '../../format';
import { ProjectLink } from '../../projects';
import { COMMAND_CLIP } from '../processes.constants';
import type { ProcessCellProps } from '../processes.type';
import { KillCell } from './KillCell';
import { NameCell } from './NameCell';
import { StatusCell } from './StatusCell';

const ProcessCell = (props: ProcessCellProps) => {
  const { row, path, kill } = props;
  const proc = row.item;
  const absent = <Text tone="muted">-</Text>;
  if (path === 'name') return <NameCell name={row.name} pid={row.pid} />;
  if (path === 'project') return proc.project ? <ProjectLink project={proc.project} /> : absent;
  if (path === 'ageMin') return formatMinutes(row.ageMin);
  if (path === 'idleMin') return formatMinutes(row.idleMin);
  if (path === 'status') return proc.status ? <StatusCell proc={proc} /> : absent;
  if (path === 'cmd') return proc.cmd ? <CopyValue value={clipText(proc.cmd, COMMAND_CLIP)} text={proc.cmd} mono size="sm" label="Command line" /> : absent;
  return <KillCell proc={proc} kill={kill} />;
};

export { ProcessCell };
