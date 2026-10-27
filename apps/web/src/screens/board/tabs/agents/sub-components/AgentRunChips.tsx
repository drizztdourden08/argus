/* @layer renderer-app @kind component */
import { Pressable, Tag } from '@drizztdourden08/tessera';
import { runLine } from '../../../../../data/runs';
import type { AgentRunChipsProps } from '../AgentsTab.type';
import { tagLook } from '../model';

const AgentRunChips = (props: AgentRunChipsProps) => {
  const { runs, onLog } = props;
  return (
    <>
      {runs.map((run) => {
        const line = runLine(run);
        return (
          <Pressable
            key={run.id}
            title={`${run.name} ${run.id}: ${run.status}${line.last ? ` · ${line.last}` : ''}`}
            onClick={(event) => {
              event.stopPropagation();
              onLog(run.id);
            }}
          >
            <Tag {...tagLook(line.tone)}>{run.name}</Tag>
          </Pressable>
        );
      })}
    </>
  );
};

export { AgentRunChips };
