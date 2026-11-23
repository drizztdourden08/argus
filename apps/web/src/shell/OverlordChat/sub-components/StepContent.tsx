/* @layer renderer-app @kind component */
import { Disclosure } from '@drizztdourden08/tessera';
import { THINKING_LABEL } from '../OverlordChat.constants';
import type { StepContentProps } from '../OverlordChat.type';
import { Muted } from './Muted';

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
