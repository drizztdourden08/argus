/* @layer renderer-app @kind component */
import { Button, CopyButton, Flex, Icon, IconButton } from '@drizztdourden08/tessera';
import { neighbours } from '../model';
import type { DrawerToolbarProps } from '../shared.type';

const linkOf = (id: string): string => `${window.location.href.split('#')[0] ?? ''}#${id}`;

const DrawerToolbar = (props: DrawerToolbarProps) => {
  const { id, order, label, wide, onWide, onOpen, children } = props;
  const { previous, next } = neighbours(order, id);
  const steps = [
    { key: 'previous', name: 'Previous in the list ([)', target: previous, icon: 'chevron-left' },
    { key: 'next', name: 'Next in the list (])', target: next, icon: 'chevron-right' },
  ] as const;
  return (
    <Flex gap="xs" align="center" wrap role="toolbar" aria-label={label}>
      <CopyButton text={() => linkOf(id)} label={`Copy a link to ${id}`} variant="ghost" size="xs" />
      {steps.map((step) => (
        <IconButton key={step.key} label={step.name} size="xs" variant="ghost" disabled={step.target === null} onClick={() => step.target && onOpen(step.target)}>
          <Icon name={step.icon} />
        </IconButton>
      ))}
      <Button size="xs" variant="ghost" aria-pressed={wide} title="Focus mode: the drawer takes the whole width" onClick={onWide}>
        {wide ? 'Collapse' : 'Expand'}
      </Button>
      {children}
    </Flex>
  );
};

export { DrawerToolbar };
