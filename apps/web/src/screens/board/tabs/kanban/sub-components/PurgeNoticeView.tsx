/* @layer renderer-app @kind component */
import { Button, Callout } from '@drizztdourden08/tessera';
import type { PurgeNoticeViewProps } from '../trash-props.type';

const PurgeNoticeView = (props: PurgeNoticeViewProps) => {
  const { notice, ops } = props;
  const dismiss = <Button size="sm" variant="ghost" onClick={() => ops.dismissPurge(notice)}>Dismiss</Button>;
  return (
    <Callout tone={notice.status === 'refused' ? 'danger' : 'success'} action={dismiss}>
      {`Purge ${notice.status} ${notice.outcome}`.trim()}
    </Callout>
  );
};

export { PurgeNoticeView };
