/* @layer renderer-app @kind component */
import { Button, CopyButton, Flex, Icon, IconButton } from '@drizztdourden08/tessera';
import type { DrawerNavProps } from '../drawer-props.type';
import { linkOf } from '../model';

const DrawerNav = (props: DrawerNavProps) => {
  const { item, ops, nav, wide, onWide } = props;
  const { previous, next, onStep } = nav;
  return (
    <Flex gap="xs" align="center" wrap role="toolbar" aria-label="Item tools">
      <CopyButton text={() => linkOf(item.id)} label={`Copy a link to ${item.id}`} variant="ghost" size="xs" />
      <IconButton label="Previous in the list ([)" size="xs" variant="ghost" disabled={previous === null} onClick={() => previous && onStep(previous)}>
        <Icon name="chevron-left" />
      </IconButton>
      <IconButton label="Next in the list (])" size="xs" variant="ghost" disabled={next === null} onClick={() => next && onStep(next)}>
        <Icon name="chevron-right" />
      </IconButton>
      <Button size="xs" variant="ghost" aria-pressed={wide} title="Focus mode: the drawer takes the whole width" onClick={onWide}>
        {wide ? 'Collapse' : 'Expand'}
      </Button>
      <Button size="xs" variant="secondary" onClick={() => void ops.soft(item.archivedAt ? 'unarchive' : 'archive', item)}>
        {item.archivedAt ? 'Unarchive' : 'Archive'}
      </Button>
      <Button size="xs" variant="danger" onClick={() => void ops.soft('delete', item)}>Delete</Button>
    </Flex>
  );
};

export { DrawerNav };
