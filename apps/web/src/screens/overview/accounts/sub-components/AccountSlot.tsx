/* @layer renderer-app @kind component */
import { Box, DropdownMenu, DropSlot } from '@drizztdourden08/tessera';
import { useRef, useState } from 'react';
import { DRAG_TYPE, SLOT_CLEAR } from '../next.constants';
import type { AccountSlotProps } from '../next.type';
import { findAccount, slotGroups } from '../model';
import { SlotName } from './SlotName';

const AccountSlot = (props: AccountSlotProps) => {
  const { label, accounts, value, disabled, onPick, onClear } = props;
  const anchor = useRef<HTMLElement>(null);
  const [open, setOpen] = useState(false);
  const chosen = findAccount(accounts, value);
  const pick = (key: string) => {
    setOpen(false);
    onPick(key);
  };
  const drop = (key: string) => {
    if (!disabled && findAccount(accounts, key)) onPick(key);
  };
  return (
    <Box ref={anchor}>
      <DropSlot
        label={label}
        size="lg"
        accept={DRAG_TYPE}
        clearLabel={SLOT_CLEAR}
        disabled={disabled}
        onDrop={drop}
        onClear={chosen ? onClear : undefined}
        onPick={() => setOpen(true)}
      >
        {chosen ? <SlotName account={chosen} /> : undefined}
      </DropSlot>
      {open && <DropdownMenu groups={slotGroups(accounts, value, pick, onClear)} label={label} anchorRef={anchor} side="below" align="start" onClose={() => setOpen(false)} />}
    </Box>
  );
};

export { AccountSlot };
