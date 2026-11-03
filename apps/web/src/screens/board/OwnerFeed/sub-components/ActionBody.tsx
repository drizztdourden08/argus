/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { LinkedText } from '../../../../shell';
import { plainText } from '../../plain-text';
import type { ActionBodyProps } from '../OwnerFeed.type';

const ActionBody = (props: ActionBodyProps) => {
  const { action, actions } = props;
  return (
    <Flex gap="sm" align="center" justify="between">
      <Text title={action.text}><LinkedText text={plainText(action.text)} /></Text>
      <Button size="sm" disabled={actions.isBusy(`post:done:${action.id}`)} onClick={() => void actions.done(action)}>Done</Button>
    </Flex>
  );
};

export { ActionBody };
