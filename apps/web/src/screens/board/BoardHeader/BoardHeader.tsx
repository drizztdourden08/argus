/* @layer renderer-app @kind component */
import { BrandMark, Flex, Stack, Text } from '@drizztdourden08/tessera';
import type { ProjectThread } from '../../../data/agents';
import { useCollection } from '../../../data/react';
import { ProjectSwitcher } from '../../../shell';
import type { BoardHeaderProps } from './BoardHeader.type';
import { findLead } from '../tabs/agents';
import { BoardStamp } from './sub-components/BoardStamp';
import { LeadStatus } from './sub-components/LeadStatus';
import './BoardHeader.css';

const BoardHeader = (props: BoardHeaderProps) => {
  const { boardKey, name, status, updatedAt, tools } = props;
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  return (
    <Flex as="header" className="board-header" wrap align="center">
      <Flex className="board-header__brand" align="center" gap="sm">
        <BrandMark app="argus" size="lg" />
        <Stack className="board-header__id" gap="2xs">
          <Text variant="overline" aria-hidden="true">Coordination board</Text>
          <Text as="h1" className="board-header__title">{name}</Text>
        </Stack>
      </Flex>
      <Flex className="board-header__status" wrap align="center">
        <LeadStatus lead={findLead(threads.docs)} />
        <BoardStamp status={status} updatedAt={updatedAt} />
      </Flex>
      <Flex className="board-header__tools" gap="xs" align="center">
        <ProjectSwitcher current={boardKey} />
        {tools}
      </Flex>
    </Flex>
  );
};

export { BoardHeader };
