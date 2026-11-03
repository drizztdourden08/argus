/* @layer renderer-app @kind component */
import { Flex, Icon, Tag, Text } from '@drizztdourden08/tessera';
import { hygieneTag } from '../../../../screens/board/tabs/worktrees/model';
import { SOURCE_TIP } from './served.constants';
import type { WorktreeLineProps } from './served.type';

const WorktreeLine = (props: WorktreeLineProps) => {
  const { row } = props;
  const hygiene = row.hygiene ? hygieneTag(row.hygiene) : null;
  const source = row.linkSource;
  return (
    <Flex gap="sm" align="center" wrap role="listitem" data-worktree={row.id}>
      <Text variant="caption" tone="dim" mono weight={600} title={row.path}>{row.path}</Text>
      {row.branch && (
        <Text variant="caption" tone="muted" title="Branch">
          <Icon name="git-fork" size={12} aria-hidden />
          {` ${row.branch}`}
        </Text>
      )}
      {hygiene ? <Tag {...hygiene.look} fill="tint" edge={hygiene.edge} title={hygiene.tip}>{hygiene.text}</Tag> : <Tag>{row.status}</Tag>}
      {Boolean(row.dirty) && <Text variant="caption" tone="muted" title="Uncommitted changes">{`${row.dirty} dirty`}</Text>}
      {Boolean(row.ahead) && <Text variant="caption" tone="muted" title="Commits ahead of the default branch">{`${row.ahead} ahead`}</Text>}
      {source && <Tag title={SOURCE_TIP[source]}>{`linked by ${source}`}</Tag>}
    </Flex>
  );
};

export { WorktreeLine };
