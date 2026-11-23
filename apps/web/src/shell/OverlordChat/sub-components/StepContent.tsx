/* @layer renderer-app @kind component */
import { Disclosure } from '@drizztdourden08/tessera';
import { tailOf } from '../behavior/tail-of';
import { THINKING_LABEL } from '../OverlordChat.constants';
import type { StepContentProps } from '../OverlordChat.type';
import { Muted } from './Muted';

const StepContent = (props: StepContentProps) => {
  const { row } = props;
  if (row.kind === 'thinking') {
    const label = row.live ? `${THINKING_LABEL}: ${tailOf(row.text)}` : THINKING_LABEL;
    return (
      <Disclosure size="sm" summary={<Muted text={label} />}>
        <Muted text={row.text} />
      </Disclosure>
    );
  }
  return <Muted text={row.text} />;
};

export { StepContent };
