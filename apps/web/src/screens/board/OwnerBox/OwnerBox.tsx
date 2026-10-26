/* @layer renderer-app @kind component */
import { MarkdownEditor } from '@drizztdourden08/tessera';
import { OWNER_TEXT_MAX } from '../../../data/posts';
import { textProblem } from '../owner-posts';
import type { OwnerBoxProps } from './OwnerBox.type';

const OwnerBox = (props: OwnerBoxProps) => {
  const { label, ariaLabel, placeholder, value, onChange, onSubmit, onEscape, problem, rows = 4, disabled = false } = props;
  return (
    <MarkdownEditor
      label={label}
      aria-label={ariaLabel}
      placeholder={placeholder}
      hint="Ctrl+Enter sends"
      error={problem === undefined ? textProblem(value) : problem}
      value={value}
      onChange={onChange}
      onSubmit={() => onSubmit()}
      onEscape={onEscape}
      maxLength={OWNER_TEXT_MAX}
      rows={rows}
      layout="tabs"
      disabled={disabled}
    />
  );
};

export { OwnerBox };
