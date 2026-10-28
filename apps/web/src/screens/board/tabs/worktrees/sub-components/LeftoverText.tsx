/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { confirmSize, filesText } from '../model';
import type { LostTextProps } from '../worktrees-props.type';

const LeftoverText = (props: LostTextProps) => {
  const { row } = props;
  const facts = row.leftover;
  const files = filesText(row) || '? files';
  const nodeModules = facts?.nodeModules ? ' (node_modules is not counted)' : '';
  const newest = facts?.newestAt ? <Age value={facts.newestAt} ago /> : 'at an unknown time';
  return (
    <Flex direction="column" gap="xs">
      <Text>
        <Text.B>Delete the leftover folder <Text.Code>{row.path}</Text.Code>?</Text.B> It holds {confirmSize(row)} in {files}{nodeModules}; its newest change was {newest}.{' '}
        <Text.B>No backup is possible: git does not know this folder.</Text.B> Argus renames it and deletes it in the background.
      </Text>
      {facts?.partial && <Text>It is too big to look through completely, so a repository deeper down would not have been seen.</Text>}
    </Flex>
  );
};

export { LeftoverText };
