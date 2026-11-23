/* @layer renderer-app @kind component */
import { Disclosure, Text } from '@drizztdourden08/tessera';
import { THINKING_LABEL } from '../OverlordChat.constants';
import type { StepContentProps } from '../OverlordChat.type';

const Muted = (props: { text: string }) => (
  <Text variant="caption" tone="muted">
    {props.text}
  </Text>
);

const StepContent = (props: StepContentProps) => {
  const { row } = props;
  if (row.kind === 'thinking') {
    return (
      <Disclosure size="sm" summary={<Muted text={THINKING_LABEL} />}>
        <Muted text={row.text} />
      </Disclosure>
    );
  }
  return <Muted text={row.text} />;
};

export { StepContent };
