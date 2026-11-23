/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';

const Muted = (props: { text: string }) => (
  <Text variant="caption" tone="muted">
    {props.text}
  </Text>
);

export { Muted };
