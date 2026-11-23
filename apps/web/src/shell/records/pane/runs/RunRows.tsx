/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { runLine } from '../../../../data/runs';
import { recordMenuGroups } from '../../behavior/record-menu-groups';
import { RecordRow } from '../../sub-components/RecordRow';
import { RUN_KIND } from './runs.constants';
import type { RunRowsProps } from './runs.type';

const RunRows = (props: RunRowsProps) => {
  const { data, runs } = props;
  const { boardKey, open, threads } = data;
  return (
    <Flex direction="column" gap="xs" role="list" aria-label="Runs">
      {runs.map((run) => (
        <RecordRow
          key={run.id}
          head={{ kind: RUN_KIND, code: run.id, assignee: run.agent ?? '', boardKey, threads, menu: recordMenuGroups(RUN_KIND, run.id, { open: () => open(run.id) }), menuLabel: `More actions for ${run.id}` }}
          title={runLine(run).last || run.name}
          chips={[{ id: 'status', text: run.status }, ...(run.exitCode === null ? [] : [{ id: 'exit', text: `exit ${run.exitCode}` }])]}
          time={run.queuedAt}
          onOpen={() => open(run.id)}
        />
      ))}
    </Flex>
  );
};

export { RunRows };
