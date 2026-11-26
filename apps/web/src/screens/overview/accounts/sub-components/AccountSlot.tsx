/* @layer renderer-app @kind component */
import { Box, DropdownMenu, DropSlot, Stack, Text } from '@drizztdourden08/tessera';
import { useRef, useState } from 'react';
import { DRAG_TYPE, SLOT_CLEAR } from '../next.constants';
import type { AccountSlotProps } from '../next.type';
import { accountKey, dragAccount, findAccount, slotGroups } from '../model';
import { AccountBrief } from './AccountBrief';
import './AccountSlot.css';

const AccountSlot = (props: AccountSlotProps) => {
  const { label, accounts, value, hint, disabled, onPick, onClear } = props;
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
    <Stack gap="2xs" align="center">
      <Box ref={anchor}>
        <DropSlot
          className="account-slot"
          label={label}
          accept={DRAG_TYPE}
          clearLabel={`${SLOT_CLEAR} ${label}`}
          disabled={disabled}
          onDrop={drop}
          onClear={chosen ? onClear : undefined}
          onPick={() => setOpen(true)}
        >
          {chosen ? <Box className="account-slot__held" draggable onDragStart={dragAccount(accountKey(chosen))}><AccountBrief account={chosen} /></Box> : undefined}
        </DropSlot>
      </Box>
      {hint && <Text variant="caption" tone="muted" clamp={1}>{hint}</Text>}
      {open && <DropdownMenu groups={slotGroups(accounts, value, pick, onClear)} label={label} anchorRef={anchor} side="below" align="start" onClose={() => setOpen(false)} />}
    </Stack>
  );
};

export { AccountSlot };
