/* @layer renderer-app @kind component */
import { Button, Flex, Text, TextInput } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { AddLineProps } from '../drawer-props.type';

const AddLine = (props: AddLineProps) => {
  const { label, limit, buttonLabel, problemOf, onAdd } = props;
  const [draft, setDraft] = useState('');
  const [problem, setProblem] = useState<string | null>(null);
  const add = () => {
    const found = problemOf(draft);
    setProblem(found);
    if (found) return;
    void onAdd(draft.trim()).then((ok) => {
      if (ok) setDraft('');
    });
  };
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="xs" align="center">
        <TextInput
          aria-label={label}
          placeholder={label}
          maxLength={limit}
          value={draft}
          onChange={(event) => {
            setProblem(null);
            setDraft(event.target.value);
          }}
          onEnter={add}
        />
        <Button size="sm" onClick={add}>{buttonLabel}</Button>
      </Flex>
      {problem && <Text variant="caption" tone="danger" role="alert">{problem}</Text>}
    </Flex>
  );
};

export { AddLine };
