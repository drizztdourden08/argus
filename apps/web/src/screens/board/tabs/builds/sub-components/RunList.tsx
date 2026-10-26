/* @layer renderer-app @kind component */
import { ListItemList } from '@drizztdourden08/tessera';
import { RUN_ROW_SHAPE } from '../builds.constants';
import type { RunListProps } from '../builds-props.type';
import { RunRow } from './RunRow';

const RunList = (props: RunListProps) => {
  const { title, runs, boardKey, onLog } = props;
  return (
    <ListItemList label={title} heading={title} count={runs.length} shape={RUN_ROW_SHAPE}>
      {runs.map((run) => <RunRow key={run.id} run={run} boardKey={boardKey} onLog={onLog} />)}
    </ListItemList>
  );
};

export { RunList };
