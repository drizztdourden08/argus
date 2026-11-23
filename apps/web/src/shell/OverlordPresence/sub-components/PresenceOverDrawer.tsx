/* @layer renderer-app @kind component */
import { Box, Portal } from '@drizztdourden08/tessera';
import { useLayoutEffect, useRef } from 'react';
import { useChatOpen } from '../../OverlordChat';
import { HEADER_MARK_SELECTOR } from '../OverlordPresence.constants';
import { OverlordPresence } from '../OverlordPresence';

const PresenceOverDrawer = () => {
  const { open } = useChatOpen();
  const box = useRef<HTMLElement>(null);
  useLayoutEffect(() => {
    const rect = document.querySelector(HEADER_MARK_SELECTOR)?.getBoundingClientRect();
    if (!open || !rect || !box.current) return;
    box.current.style.setProperty('--float-top', `${rect.top}px`);
    box.current.style.setProperty('--float-left', `${rect.left}px`);
  }, [open]);
  if (!open) return null;
  return (
    <Portal layer="popover">
      <Box ref={box} className="overlord-presence-float">
        <OverlordPresence />
      </Box>
    </Portal>
  );
};

export { PresenceOverDrawer };
