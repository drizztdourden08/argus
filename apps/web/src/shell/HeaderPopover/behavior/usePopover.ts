/* @layer renderer-app @kind logic */
import { useCallback, useRef, useState } from 'react';
import type { PopoverView } from '../HeaderPopover.type';

const usePopover = (): PopoverView => {
  const anchor = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const toggle = useCallback(() => setOpen((was) => !was), []);
  const close = useCallback(() => setOpen(false), []);
  return { anchor, open, toggle, close };
};

export { usePopover };
