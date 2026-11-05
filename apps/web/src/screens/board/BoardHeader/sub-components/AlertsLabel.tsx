/* @layer renderer-app @kind component */
import { Badge } from '@drizztdourden08/tessera';
import type { AlertsLabelProps } from '../BoardHeader.type';

const AlertsLabel = (props: AlertsLabelProps) => {
  const { count, tone } = props;
  return (
    <>
      Alerts {count > 0 ? <Badge variant="number" color={tone} value={count} label={`${count} alerts`} /> : null}
    </>
  );
};

export { AlertsLabel };
