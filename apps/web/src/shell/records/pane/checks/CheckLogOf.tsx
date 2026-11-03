/* @layer renderer-app @kind component */
import type { RecordBodyProps } from '../pane.type';
import { CheckLog } from './CheckLog';
import { useCheckRun } from './useCheckRun';

const CheckLogOf = (props: RecordBodyProps) => {
  const { data } = props;
  const { run, ready } = useCheckRun(data.boardKey, data.target.id);
  return <CheckLog data={data} run={run} ready={ready} />;
};

export { CheckLogOf };
