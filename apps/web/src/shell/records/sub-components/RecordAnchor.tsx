/* @layer renderer-app @kind component */
import { Link } from '@drizztdourden08/tessera';
import { useRecordTarget } from '../behavior/useRecordTarget';
import type { RecordAnchorProps } from '../records.type';
import './RecordAnchor.css';

const RecordAnchor = (props: RecordAnchorProps) => {
  const { target, label, title } = props;
  const go = useRecordTarget(target);
  if (!go) return <>{label}</>;
  return (
    <Link href={go.href} navigate={go.go} tone="neutral" variant="subtle" title={title} className="record-anchor">
      {label}
    </Link>
  );
};

export { RecordAnchor };
