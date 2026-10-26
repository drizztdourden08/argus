/* @layer renderer-app @kind component */
import { MarkdownEditor } from '@drizztdourden08/tessera';
import { textProblem } from '../owner-posts';
import type { OwnerBoxProps } from './OwnerBox.type';

const OwnerBox = (props: OwnerBoxProps) => {
  const { label, value, onChange, onSubmit, rows = 4, disabled = false } = props;
  return (
    <MarkdownEditor
      label={label}
      hint="Ctrl+Enter sends"
      error={textProblem(value)}
      value={value}
      onChange={onChange}
      onSubmit={() => onSubmit()}
      rows={rows}
      layout="tabs"
      disabled={disabled}
    />
  );
};

export { OwnerBox };
