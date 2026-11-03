/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import type { CapToggleProps } from '../worktrees-props.type';

const CapToggle = (props: CapToggleProps) => {
  const { total, limit, all, onToggle } = props;
  if (total <= limit) return null;
  return (
    <Flex gap="sm" align="center">
      <Text variant="caption" tone="muted">{all ? `All ${total} shown` : `Latest ${limit} of ${total}`}</Text>
      <Button size="sm" variant="ghost" onClick={onToggle}>{all ? `Show the latest ${limit}` : `Show all ${total}`}</Button>
    </Flex>
  );
};

export { CapToggle };
