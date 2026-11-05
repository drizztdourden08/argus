/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import type { TreeFoldBarProps } from '../tree.type';

const TreeFoldBar = (props: TreeFoldBarProps) => {
  const { fold } = props;
  return (
    <Flex gap="sm" align="center">
      <Button size="sm" variant="ghost" onClick={fold.unfoldAll}>Unfold all</Button>
      <Button size="sm" variant="ghost" onClick={fold.foldAll}>Fold all</Button>
    </Flex>
  );
};

export { TreeFoldBar };
