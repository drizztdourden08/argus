/* @layer renderer-app @kind component */
import { useCheckRun } from '../checks/useCheckRun';
import { CheckLog } from '../checks/CheckLog';
import type { RecordBodyProps } from '../pane.type';
import { RunLog } from '../runs/RunLog';

const CheckLogOf = (props: RecordBodyProps) => {
  const { data } = props;
  const { run, ready } = useCheckRun(data.boardKey, data.target.id);
  return <CheckLog data={data} run={run} ready={ready} />;
};

const LogTab = (props: RecordBodyProps) => (props.data.target.collection === 'checks' ? <CheckLogOf {...props} /> : <RunLog {...props} />);

export { LogTab };
