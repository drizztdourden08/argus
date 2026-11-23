/* @layer renderer-app @kind logic */
import type { ToolItem, ToolStatus } from '../OverlordChat.type';

const countStatus = (tools: readonly ToolItem[], status: ToolStatus): number => tools.filter((tool) => tool.status === status).length;

export { countStatus };
