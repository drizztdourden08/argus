/* @layer renderer-app @kind logic */
import { useRef, useState } from 'react';

const useUsagePopover = (reload: () => void) => {
  const anchor = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  const toggle = () => {
    if (!open) reload();
    setOpen(!open);
  };

  return { anchor, open, toggle, close: () => setOpen(false) };
};

export { useUsagePopover };
