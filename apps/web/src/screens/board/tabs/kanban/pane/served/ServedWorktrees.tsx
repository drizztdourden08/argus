/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../sub-components/DrawerSection';
import { EmptyLine } from '../../sub-components/EmptyLine';
import { NO_WORKTREES_TEXT, WORKTREES_TITLE } from './served.constants';
import type { ServedWorktreesProps } from './served.type';
import { WorktreeLine } from './WorktreeLine';

const ServedWorktrees = (props: ServedWorktreesProps) => {
  const { rows } = props;
  return (
    <DrawerSection title={`${WORKTREES_TITLE} · ${rows.length}`}>
      {rows.length === 0 && <EmptyLine>{NO_WORKTREES_TEXT}</EmptyLine>}
      <Flex direction="column" gap="xs" role="list" aria-label={WORKTREES_TITLE}>
        {rows.map((row) => <WorktreeLine key={row.id} row={row} />)}
      </Flex>
    </DrawerSection>
  );
};

export { ServedWorktrees };
