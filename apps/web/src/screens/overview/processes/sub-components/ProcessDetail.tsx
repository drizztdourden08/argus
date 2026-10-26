/* @layer renderer-app @kind component */
import { Card, FactsPanel, Stack, Tag } from '@drizztdourden08/tessera';
import { clipText, formatCpu, formatMemory, formatMinutes } from '../../format';
import { ProjectLink } from '../../projects';
import { procLabel, STATUS_COLOR, statusTip } from '../model';
import { COMMAND_CLIP } from '../processes.constants';
import type { ProcessDetailProps } from '../processes.type';
import { ProcessKill } from './ProcessKill';

const ProcessDetail = (props: ProcessDetailProps) => {
  const { proc, kill } = props;
  const status = String(proc.status ?? '');
  const command = proc.cmd ?? '';
  const facts = [[
    { label: 'Rule', value: proc.rule ?? '-' },
    { label: 'Status', value: status ? <Tag variant="urgency" color={STATUS_COLOR[status] ?? 'info'}>{status}</Tag> : '-', title: statusTip(proc) },
    { label: 'Project', value: proc.project ? <ProjectLink project={proc.project} /> : '-' },
    { label: 'Memory', value: formatMemory(proc.memMB) },
    { label: 'CPU', value: formatCpu(proc.cpuPct) },
    { label: 'Age', value: formatMinutes(proc.ageMin) },
    { label: 'Idle', value: formatMinutes(proc.idleMin) },
    { label: 'Command', value: command ? clipText(command, COMMAND_CLIP) : '-', mono: true, title: command, copyable: command || false },
  ]];
  return (
    <Card title={procLabel(proc)} subtitle={proc.hint ?? undefined}>
      <Stack gap="md">
        <FactsPanel groups={facts} layout="rows" label="Process details" />
        <ProcessKill proc={proc} kill={kill} />
      </Stack>
    </Card>
  );
};

export { ProcessDetail };
