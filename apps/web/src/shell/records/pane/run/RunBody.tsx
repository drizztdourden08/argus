/* @layer renderer-app @kind component */
import { RunStage } from './RunStage';
import { RunStart } from './RunStart';
import { RunSummary } from './RunSummary';
import type { RunBodyProps } from './run.type';

const RunBody = (props: RunBodyProps) => {
  const { data, run, ids, ops } = props;
  if (!run) return <RunStart ops={ops} again={false} />;
  if (run.endedAt === null) return <RunStage key={run.n} data={data} run={run} ids={ids} ops={ops} />;
  return <RunSummary data={data} run={run} ids={ids} ops={ops} />;
};

export { RunBody };
