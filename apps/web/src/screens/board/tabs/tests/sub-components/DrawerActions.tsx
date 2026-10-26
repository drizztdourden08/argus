/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { copyText } from '../../../shared';
import { useTestsShared } from '../behavior/useTestsShared';
import type { DrawerActionsProps } from '../tests-drawer-props.type';

const linkTo = (testId: string): string => `${window.location.origin}${window.location.pathname}${window.location.search}#${testId}`;

const DrawerActions = (props: DrawerActionsProps) => {
  const { test, nav, wide } = props;
  const { ops, onEdit } = useTestsShared();
  return (
    <>
      <Button size="sm" variant="ghost" disabled={nav.previous === null} aria-label="Previous in the list ([)" onClick={() => nav.step(-1)}>Previous</Button>
      <Button size="sm" variant="ghost" disabled={nav.next === null} aria-label="Next in the list (])" onClick={() => nav.step(1)}>Next</Button>
      <Button size="sm" variant="ghost" aria-label={`Copy a link to ${test.id}`} onClick={() => copyText(linkTo(test.id))}>Copy link</Button>
      <Button size="sm" variant="ghost" aria-pressed={wide.wide} title="Focus mode: the drawer takes the whole width" onClick={wide.toggle}>
        {wide.wide ? 'Collapse' : 'Expand'}
      </Button>
      <Button size="sm" variant="secondary" onClick={() => onEdit(test.id)}>Edit</Button>
      <Button size="sm" variant="secondary" onClick={() => void ops.soft(test.archivedAt ? 'unarchive' : 'archive', test)}>
        {test.archivedAt ? 'Unarchive' : 'Archive'}
      </Button>
      <Button size="sm" variant="danger" onClick={() => void ops.soft('delete', test)}>Delete</Button>
    </>
  );
};

export { DrawerActions };
