/* @layer renderer-app @kind component */
import { Flex, Text, Textarea } from '@drizztdourden08/tessera';
import type { KeyboardEvent } from 'react';
import { textProblem } from '../owner-posts';
import type { OwnerBoxProps } from './OwnerBox.type';

const OwnerBox = (props: OwnerBoxProps) => {
  const { label, value, onChange, onSubmit, rows = 2, disabled = false } = props;
  const problem = textProblem(value);
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    onSubmit();
  };
  return (
    <Flex direction="column" gap="2xs">
      <Textarea
        aria-label={label}
        placeholder={label}
        rows={rows}
        value={value}
        disabled={disabled}
        invalid={problem !== null}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={onKeyDown}
      />
      <Text variant="caption" tone="muted">Ctrl+Enter sends</Text>
      {problem ? <Text variant="caption" tone="danger" role="alert">{problem}</Text> : null}
    </Flex>
  );
};

export { OwnerBox };
