/* @layer renderer-app @kind component */
import { Button, FilterBar, Flex, SegmentedControl } from '@drizztdourden08/tessera';
import { FILTER_LABEL } from '../AgentsTab.constants';
import type { AgentsToolbarProps } from '../AgentsTab.type';
import type { AgentFilter } from '../model';

const AgentsToolbar = (props: AgentsToolbarProps) => {
  const { query, onQuery, filter, onFilter, runningCount, doneCount, onArchive } = props;
  const control = (
    <Flex gap="sm" align="center">
      <SegmentedControl<AgentFilter>
        label={FILTER_LABEL}
        size="sm"
        value={filter}
        onChange={onFilter}
        options={[
          { value: 'running', label: `Running (${runningCount})` },
          { value: 'done', label: `Done (${doneCount})` },
          { value: 'all', label: 'All' },
        ]}
      />
      {onArchive && <Button size="sm" variant="secondary" onClick={onArchive}>Archive finished</Button>}
    </Flex>
  );
  return (
    <FilterBar
      search={query}
      onSearchChange={onQuery}
      searchLabel="Search agents"
      searchPlaceholder="Search name, task, item or note"
      extra={control}
    />
  );
};

export { AgentsToolbar };
