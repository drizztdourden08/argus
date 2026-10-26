/* @layer renderer-app @kind component */
import { Button, Callout, Flex } from '@drizztdourden08/tessera';
import { STALE_TEXT } from '../worktrees.constants';
import type { ConfirmFooterProps } from '../worktrees-props.type';

const ConfirmFooter = (props: ConfirmFooterProps) => {
  const { stale, error, label, danger, disabled, onConfirm, onCancel } = props;
  return (
    <>
      {stale && <Callout tone="warning">{STALE_TEXT}</Callout>}
      {error && <Callout tone="danger">{error}</Callout>}
      <Flex gap="xs">
        <Button size="sm" variant={danger ? 'danger' : 'primary'} disabled={disabled || stale} onClick={onConfirm}>{label}</Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>Cancel</Button>
      </Flex>
    </>
  );
};

export { ConfirmFooter };
