/* @layer renderer-app @kind component */
import { Anchored, Button, Card } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import type { ColumnAddProps } from '../kanban-props.type';

const ColumnAdd = (props: ColumnAddProps) => {
  const { title, open, onToggle, onClose, children } = props;
  const anchor = useRef<HTMLButtonElement>(null);
  return (
    <>
      <Button ref={anchor} size="xs" variant="ghost" aria-label={`Add a work item to ${title}`} aria-expanded={open} onClick={onToggle}>
        + Add
      </Button>
      {open && (
        <Anchored anchorRef={anchor} placement="bottom-end" layer="overlay" onDismiss={onClose}>
          <Card>{children}</Card>
        </Anchored>
      )}
    </>
  );
};

export { ColumnAdd };
