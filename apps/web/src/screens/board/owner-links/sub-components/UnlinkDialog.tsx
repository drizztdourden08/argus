/* @layer renderer-app @kind component */
import { Callout, Dialog, Flex } from '@drizztdourden08/tessera';
import { linkWorktree, unlinkLauncher } from '../../../../data/posts';
import { useOwnerSend } from '../behavior/useOwnerSend';
import { BUILD_UNLINK_NOTE, WORKTREE_UNLINK_NOTE } from '../owner-links.constants';
import type { OwnerDialogProps } from '../owner-links.type';

const UnlinkDialog = (props: OwnerDialogProps) => {
  const { boardKey, ask, onClose } = props;
  const sender = useOwnerSend(onClose);
  const forBuild = ask.action === 'launcher-unlink';
  const post = () => (forBuild ? unlinkLauncher(boardKey, ask.id) : linkWorktree(boardKey, ask.id, null));
  return (
    <Dialog
      open
      size="md"
      title={`Unlink ${ask.label} from its item`}
      message={forBuild ? BUILD_UNLINK_NOTE : WORKTREE_UNLINK_NOTE}
      confirmLabel="Unlink"
      confirmDisabled={sender.busy}
      onConfirm={() => void sender.send(post, `${ask.label} is unlinked.`)}
      onCancel={onClose}
    >
      <Flex direction="column" gap="sm">
        {sender.problem && <Callout tone="danger">{sender.problem}</Callout>}
      </Flex>
    </Dialog>
  );
};

export { UnlinkDialog };
