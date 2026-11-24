/* @layer renderer-app @kind component */
import { Box, Card, DropdownMenu, Text } from '@drizztdourden08/tessera';
import type { DragEvent } from 'react';
import { useRef, useState } from 'react';
import { DRAG_TYPE, SLOT_EMPTY } from '../next.constants';
import type { AccountSlotProps } from '../next.type';
import { accountName, findAccount, slotGroups } from '../model';
import { AccountName } from './AccountName';

const AccountSlot = (props: AccountSlotProps) => {
  const { label, accounts, value, disabled, onPick, onClear } = props;
  const anchor = useRef<HTMLElement>(null);
  const [open, setOpen] = useState(false);
  const chosen = findAccount(accounts, value);
  const pick = (key: string) => {
    setOpen(false);
    onPick(key);
  };
  const drop = (event: DragEvent) => {
    event.preventDefault();
    const key = event.dataTransfer.getData(DRAG_TYPE);
    if (!disabled && findAccount(accounts, key)) onPick(key);
  };
  return (
    <Box ref={anchor}>
      <Card
        aria-label={`${label}: ${chosen ? accountName(accounts, chosen.id) : SLOT_EMPTY}`}
        onActivate={disabled ? undefined : () => setOpen(true)}
        onDragOver={(event) => event.preventDefault()}
        onDrop={drop}
      >
        {chosen ? <AccountName account={chosen} /> : <Text tone="muted">{SLOT_EMPTY}</Text>}
      </Card>
      {open && <DropdownMenu groups={slotGroups(accounts, value, pick, onClear)} label={label} anchorRef={anchor} side="below" align="start" onClose={() => setOpen(false)} />}
    </Box>
  );
};

export { AccountSlot };
