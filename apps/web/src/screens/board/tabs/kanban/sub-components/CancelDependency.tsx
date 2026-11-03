/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { useCancelDep } from '../behavior/useCancelDep';
import type { CancelDependencyProps } from '../drawer-props.type';

const CancelDependency = (props: CancelDependencyProps) => {
  const { dep } = props;
  const { busy, cancel } = useCancelDep(dep);
  return <Button size="sm" variant="ghost" loading={busy} onClick={() => void cancel()} title={`Cancel the request ${dep}`}>Cancel</Button>;
};

export { CancelDependency };
