/* @layer renderer-app @kind component */
import { AgentWorkBody } from '../../../../screens/board/tabs/agents/pane/AgentWorkBody';
import { itemBody } from '../item-body';
import { ServedTab } from './ServedTab';

const WorkTab = itemBody(ServedTab, AgentWorkBody);

export { WorkTab };
