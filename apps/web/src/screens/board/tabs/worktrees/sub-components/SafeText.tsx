/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { LostTextProps } from '../worktrees-props.type';

const SafeText = (props: LostTextProps) => {
  const { row } = props;
  const kept = row.branch && !row.detached
    ? <>and its commits stay in the branch <Text.Code>{row.branch}</Text.Code></>
    : 'and a branch keeps its commits';
  return (
    <Text>
      Remove <Text.Code>{row.path}</Text.Code> now? It is clean {kept}: nothing is lost. The branch stays unless you tick the box and it is merged.
    </Text>
  );
};

export { SafeText };
