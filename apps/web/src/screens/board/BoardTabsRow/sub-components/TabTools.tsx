/* @layer renderer-app @kind component */
import { createPortal } from 'react-dom';
import { toolsSlot, useSlotNode } from '../behavior/slot-store';
import type { PortalProps } from '../BoardTabsRow.type';

const TabTools = ({ children }: PortalProps) => {
  const node = useSlotNode(toolsSlot);
  return node ? createPortal(children, node) : null;
};

export { TabTools };
