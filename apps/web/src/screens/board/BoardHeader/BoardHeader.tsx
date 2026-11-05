/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { ProjectThread } from '../../../data/agents';
import { useCollection, useDoc } from '../../../data/react';
import type { BoardMeta } from '../../../data/types';
import { BoardIcon } from '../../../shell/BoardIcon';
import { findLead } from '../tabs/agents/model';
import { BoardSettingsButton } from '../BoardSettings';
import { RegisteredStrip } from '../RegisteredStrip';
import type { BoardHeaderProps } from './BoardHeader.type';
import { BoardAlerts } from './sub-components/BoardAlerts';
import { BoardFacts } from './sub-components/BoardFacts';
import { LeadStatus } from './sub-components/LeadStatus';
import './BoardHeader.css';

const BoardHeader = (props: BoardHeaderProps) => {
  const { boardKey, name, status, updatedAt, tools } = props;
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  const { doc } = useDoc<BoardMeta>(boardKey, 'meta', 'board');
  return (
    <Flex as="header" className="board-header" wrap align="center">
      <BoardIcon size="md" boardKey={boardKey} icon={doc?.icon} avatar={doc?.avatar} />
      <Text as="h1" className="board-header__title">{name}</Text>
      <LeadStatus lead={findLead(threads.docs)} />
      <BoardFacts className="board-header__facts" boardKey={boardKey} status={status} updatedAt={updatedAt} />
      <RegisteredStrip boardKey={boardKey} folded={<BoardFacts boardKey={boardKey} status={status} updatedAt={updatedAt} />} />
      <BoardAlerts boardKey={boardKey} status={status} />
      <Flex className="board-header__tools" gap="xs" align="center">
        <BoardSettingsButton boardKey={boardKey} />
        {tools}
      </Flex>
    </Flex>
  );
};

export { BoardHeader };
