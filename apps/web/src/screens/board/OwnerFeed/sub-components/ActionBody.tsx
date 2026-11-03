/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import type { ActionBodyProps } from '../OwnerFeed.type';

const ActionBody = (props: ActionBodyProps) => {
  const { action, actions } = props;
  return (
    <Flex gap="sm" align="center">
      <Button size="sm" disabled={actions.isBusy(`post:done:${action.id}`)} onClick={() => void actions.done(action)}>Done</Button>
    </Flex>
  );
};

export { ActionBody };
