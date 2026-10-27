/* @layer renderer-app @kind component */
import { Box, Text, TextInput } from '@drizztdourden08/tessera';
import { useRef, useState } from 'react';
import type { AgentNoteEditProps } from '../row-props.type';
import { AGENT_LIMITS, agentName, agentProblem } from '../model';

const AgentNoteEdit = (props: AgentNoteEditProps) => {
  const { id, agent, actions, onDone } = props;
  const [draft, setDraft] = useState(agent.note ?? '');
  const problem = agentProblem('note', draft);
  const finished = useRef(false);
  const commit = () => {
    if (problem || finished.current) return;
    finished.current = true;
    if (draft.trim() !== (agent.note ?? '')) actions.setField(id, 'note', draft.trim());
    onDone();
  };
  return (
    <Box role="row" className="agents-edit">
      <Box role="cell">
        <Text as="p" variant="caption">{`Note for ${agentName(agent, id)}`}</Text>
        <TextInput
          aria-label="Note"
          value={draft}
          placeholder="add a note"
          maxLength={AGENT_LIMITS.note}
          invalid={problem !== null}
          autoFocus
          grow
          size="sm"
          onChange={(event) => setDraft(event.currentTarget.value)}
          onBlur={commit}
          onKeyDown={(event) => {
            if (event.key === 'Enter') commit();
            if (event.key === 'Escape') {
              finished.current = true;
              onDone();
            }
          }}
        />
        {problem && <Text as="p" variant="caption" tone="danger">{problem}</Text>}
      </Box>
    </Box>
  );
};

export { AgentNoteEdit };
