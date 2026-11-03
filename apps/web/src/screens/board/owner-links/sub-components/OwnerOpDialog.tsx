/* @layer renderer-app @kind component */
import type { OwnerDialogProps } from '../owner-links.type';
import { KeepDialog } from './KeepDialog';
import { LinkDialog } from './LinkDialog';
import { RetireDialog } from './RetireDialog';

const OwnerOpDialog = (props: OwnerDialogProps) => {
  const { action } = props.ask;
  if (action === 'worktree-keep') return <KeepDialog {...props} />;
  if (action === 'launcher-retire') return <RetireDialog {...props} />;
  return <LinkDialog {...props} />;
};

export { OwnerOpDialog };
