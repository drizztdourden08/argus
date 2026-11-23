/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { sortRuns } from '../../../../screens/board/tabs/changes';
import { shortCommit } from '../../../../format';
import { recordMenuGroups } from '../../behavior/record-menu-groups';
import { RecordRow } from '../../sub-components/RecordRow';
import { CHECK_KIND, STATUS_TEXT } from './checks.constants';
import type { CheckRowsProps } from './checks.type';

const CheckRows = (props: CheckRowsProps) => {
  const { data, runs } = props;
  const { boardKey, open, threads } = data;
  return (
    <Flex direction="column" gap="xs" role="list" aria-label="Check runs">
      {sortRuns(runs).map((run) => (
        <RecordRow
          key={run.id}
          head={{ kind: CHECK_KIND, code: run.id, assignee: '', boardKey, threads, menu: recordMenuGroups(CHECK_KIND, run.id, { open: () => open(run.id) }), menuLabel: `More actions for ${run.id}` }}
          title={`${run.trigger} on ${shortCommit(run.commit)}${run.summary ? `: ${run.summary}` : ''}`}
          chips={[{ id: 'status', text: STATUS_TEXT[run.status] ?? run.status }, { id: 'steps', text: `${run.steps.length} steps` }]}
          time={run.startedAt}
          onOpen={() => open(run.id)}
        />
      ))}
    </Flex>
  );
};

export { CheckRows };
