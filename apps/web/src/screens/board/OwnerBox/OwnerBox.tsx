/* @layer renderer-app @kind component */
import { Flex, MarkdownEditor } from '@drizztdourden08/tessera';
import type { KeyboardEvent } from 'react';
import { textProblem } from '../owner-posts';
import type { OwnerBoxProps } from './OwnerBox.type';

const OwnerBox = (props: OwnerBoxProps) => {
  const { label, value, onChange, onSubmit, rows = 4, disabled = false } = props;
  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    onSubmit();
  };
  return (
    <Flex direction="column" onKeyDown={onKeyDown}>
      <MarkdownEditor
        label={label}
        hint="Ctrl+Enter sends"
        error={textProblem(value)}
        value={value}
        onChange={onChange}
        rows={rows}
        layout="tabs"
        disabled={disabled}
      />
    </Flex>
  );
};

export { OwnerBox };
