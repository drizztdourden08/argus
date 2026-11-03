/* @layer renderer-app @kind component */
import { Flex, Text, Textarea } from '@drizztdourden08/tessera';
import { useEffect, useRef } from 'react';
import { NOTE_MAX } from './run.constants';
import type { NoteFieldProps } from './run.type';

const NoteField = (props: NoteFieldProps) => {
  const { value, problem, disabled, onChange } = props;
  const field = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    if (problem) field.current?.focus();
  }, [problem]);
  return (
    <Flex direction="column" gap="xs">
      <Textarea
        ref={field}
        size="sm"
        rows={2}
        aria-label="Note"
        placeholder="Note (required for fail, partial and blocked)"
        maxLength={NOTE_MAX}
        invalid={problem !== null}
        disabled={disabled}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
      {problem && <Text variant="caption" tone="danger" role="alert">{problem}</Text>}
    </Flex>
  );
};

export { NoteField };
