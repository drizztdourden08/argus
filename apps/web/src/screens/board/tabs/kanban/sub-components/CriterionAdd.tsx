/* @layer renderer-app @kind component */
import { Button, Flex, Text, TextInput } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { CriterionAddProps } from '../kanban-props.type';
import { LIMITS, textProblem } from '../model';

const CriterionAdd = (props: CriterionAddProps) => {
  const { item, ops } = props;
  const [draft, setDraft] = useState('');
  const [problem, setProblem] = useState<string | null>(null);
  const add = () => {
    const found = textProblem('criterion', draft);
    setProblem(found);
    if (found) return;
    void ops.criterion(item, { do: 'add', text: draft.trim() }, `Criterion added to ${item.id}.`).then((ok) => {
      if (ok) setDraft('');
    });
  };
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="xs" align="center">
        <TextInput
          aria-label="Add a criterion"
          placeholder="Add a criterion"
          maxLength={LIMITS.criterion}
          value={draft}
          onChange={(event) => {
            setProblem(null);
            setDraft(event.target.value);
          }}
          onEnter={add}
        />
        <Button size="sm" onClick={add}>Add criterion</Button>
      </Flex>
      {problem && <Text variant="caption" tone="danger" role="alert">{problem}</Text>}
    </Flex>
  );
};

export { CriterionAdd };
