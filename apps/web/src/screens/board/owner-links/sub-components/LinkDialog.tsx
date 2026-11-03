/* @layer renderer-app @kind component */
import { Callout, Dialog, Flex, Select } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { itemTypeOf } from '../../../../data/registry';
import { useCollection } from '../../../../data/react';
import type { Item } from '../../../../data/kanban';
import { linkLauncher, linkWorktree } from '../../../../data/posts';
import { useOwnerSend } from '../behavior/useOwnerSend';
import { openItemOptions } from '../model';
import { BUILD_KIND, BUILD_LINK_NOTE, NO_CHOICE, WORKTREE_LINK_NOTE } from '../owner-links.constants';
import type { OwnerDialogProps } from '../owner-links.type';

const LinkDialog = (props: OwnerDialogProps) => {
  const { boardKey, ask, onClose } = props;
  const items = useCollection<Item>(boardKey, 'items');
  const [item, setItem] = useState(NO_CHOICE);
  const sender = useOwnerSend(onClose);
  const forBuild = ask.action === 'launcher-link';
  const serves = forBuild ? itemTypeOf(BUILD_KIND)?.serves : undefined;
  const options = useMemo(() => openItemOptions(items.docs, serves, 'Choose an open item'), [items.docs, serves]);
  const post = () => (forBuild ? linkLauncher(boardKey, ask.id, item) : linkWorktree(boardKey, ask.id, item));
  return (
    <Dialog
      open
      size="md"
      title={`Link ${ask.label} to an item`}
      message={forBuild ? BUILD_LINK_NOTE : WORKTREE_LINK_NOTE}
      confirmLabel="Link"
      confirmDisabled={sender.busy || item === NO_CHOICE}
      onConfirm={() => void sender.send(post, `${ask.label} is linked to ${item}.`)}
      onCancel={onClose}
    >
      <Flex direction="column" gap="sm">
        <Select aria-label="Item" options={options} value={item} onChange={setItem} />
        {sender.problem && <Callout tone="danger">{sender.problem}</Callout>}
      </Flex>
    </Dialog>
  );
};

export { LinkDialog };
