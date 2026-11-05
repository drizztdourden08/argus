/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import type { AvatarResetProps } from './AvatarReset.type';

const AvatarReset = (props: AvatarResetProps) => {
  const { disabled, problem, onReset } = props;
  return (
    <Flex gap="sm" align="center">
      <Button size="sm" variant="secondary" disabled={disabled} onClick={onReset}>Use the default</Button>
      {problem && <Text variant="caption" tone="danger" role="alert">{problem}</Text>}
    </Flex>
  );
};

export { AvatarReset };
