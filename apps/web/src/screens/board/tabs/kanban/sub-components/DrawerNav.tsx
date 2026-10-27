/* @layer renderer-app @kind component */
import { Button, CopyButton, Icon, IconButton } from '@drizztdourden08/tessera';
import type { DrawerNavProps } from '../drawer-props.type';
import { linkOf } from '../model';

const DrawerNav = (props: DrawerNavProps) => {
  const { item, ops, nav, wide, onWide } = props;
  const { previous, next, onStep } = nav;
  const archived = Boolean(item.archivedAt);
  return (
    <>
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
      <IconButton
        label={archived ? 'Unarchive' : 'Archive'}
        size="xs"
        variant="ghost"
        onClick={() => void ops.soft(archived ? 'unarchive' : 'archive', item)}
      >
        <Icon name={archived ? 'rotate-ccw' : 'archive'} />
      </IconButton>
      <IconButton label="Delete" title="Delete (Undo stays a few seconds)" size="xs" variant="ghost" tone="danger" onClick={() => void ops.soft('delete', item)}>
        <Icon name="trash-2" />
      </IconButton>
    </>
  );
};

export { DrawerNav };
