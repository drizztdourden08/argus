/* @layer renderer-app @kind component */
import { Link } from '@drizztdourden08/tessera';
import { useRecordTarget } from '../behavior/useRecordTarget';
import type { RecordAnchorProps } from '../records.type';

const RecordAnchor = (props: RecordAnchorProps) => {
  const { target, label, title } = props;
  const go = useRecordTarget(target);
  if (!go) return <>{label}</>;
  return (
    <Link href={go.href} navigate={go.go} tone="neutral" variant="subtle" title={title}>
      {label}
    </Link>
  );
};

export { RecordAnchor };
