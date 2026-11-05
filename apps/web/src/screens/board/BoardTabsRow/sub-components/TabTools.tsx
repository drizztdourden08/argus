/* @layer renderer-app @kind component */
import { createPortal } from 'react-dom';
import { toolsSlot } from '../behavior/tools-slot';
import { useSlotNode } from '../behavior/useSlotNode';
import type { PortalProps } from '../BoardTabsRow.type';

const TabTools = ({ children }: PortalProps) => {
  const node = useSlotNode(toolsSlot);
  return node ? createPortal(children, node) : null;
};

export { TabTools };
