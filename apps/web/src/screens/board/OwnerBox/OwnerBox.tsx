/* @layer renderer-app @kind component */
import { BoardMarkdownEditor } from '../../../shell';
import { OWNER_TEXT_MAX } from '../../../data/posts';
import { textProblem } from '../owner-posts';
import type { OwnerBoxProps } from './OwnerBox.type';

const OwnerBox = (props: OwnerBoxProps) => {
  const { label, ariaLabel, placeholder, value, onChange, onSubmit, onEscape, problem, rows = 4, disabled = false, autoFocus = false, compact = false, actions, submitLabel } = props;
  return (
    <BoardMarkdownEditor
      label={label}
      aria-label={ariaLabel}
      placeholder={placeholder}
      error={problem === undefined ? textProblem(value) : problem}
      value={value}
      onChange={onChange}
      onSubmit={() => onSubmit()}
      onEscape={onEscape}
      actions={actions}
      submitLabel={submitLabel}
      maxLength={OWNER_TEXT_MAX}
      rows={rows}
      layout="tabs"
      compact={compact}
      disabled={disabled}
      autoFocus={autoFocus}
    />
  );
};

export { OwnerBox };
