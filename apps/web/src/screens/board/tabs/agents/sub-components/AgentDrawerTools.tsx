/* @layer renderer-app @kind component */
import { Button, CopyButton, Flex, Icon, IconButton } from '@drizztdourden08/tessera';
import type { AgentDrawerToolsProps } from '../AgentsTab.type';
import { neighbours } from '../../../shared';

const linkOf = (id: string): string => `${window.location.href.split('#')[0] ?? ''}#${id}`;

const AgentDrawerTools = (props: AgentDrawerToolsProps) => {
  const { id, agent, order, actions, wide, onWide, onOpen } = props;
  const { previous, next } = neighbours(order, id);
  return (
    <Flex gap="xs" align="center" wrap role="toolbar" aria-label="Agent tools">
      <CopyButton text={() => linkOf(id)} label={`Copy a link to ${id}`} variant="ghost" size="xs" />
      <IconButton label="Previous in the list ([)" size="xs" variant="ghost" disabled={previous === null} onClick={() => previous && onOpen(previous)}>
        <Icon name="chevron-left" />
      </IconButton>
      <IconButton label="Next in the list (])" size="xs" variant="ghost" disabled={next === null} onClick={() => next && onOpen(next)}>
        <Icon name="chevron-right" />
      </IconButton>
      <Button size="xs" variant="ghost" aria-pressed={wide} title="Focus mode: the drawer takes the whole width" onClick={onWide}>
        {wide ? 'Collapse' : 'Expand'}
      </Button>
      <IconButton label={agent.archivedAt ? 'Unarchive this agent' : 'Archive this agent'} size="xs" variant="ghost" onClick={() => actions.soft(agent.archivedAt ? 'unarchive' : 'archive', id)}>
        <Icon name="archive" />
      </IconButton>
      <IconButton label="Delete this agent" size="xs" variant="ghost" className="agents-icon agents-icon--danger" onClick={() => actions.soft('delete', id)}>
        <Icon name="trash-2" />
      </IconButton>
    </Flex>
  );
};

export { AgentDrawerTools };
