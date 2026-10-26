/* @layer renderer-app @kind component */
import { Button, Callout, Flex } from '@drizztdourden08/tessera';
import type { OpNoticeProps } from '../kanban-props.type';
import { NoticeDiff } from './NoticeDiff';

const OpNotice = (props: OpNoticeProps) => {
  const { notice, ops } = props;
  const target = notice.op.target?.id ?? notice.op.target?.c ?? '';
  if (notice.kind === 'refused') {
    const dismiss = <Button size="sm" variant="ghost" onClick={() => ops.dismiss(notice)}>Dismiss</Button>;
    return (
      <Callout tone="danger" action={dismiss}>
        {`Refused ${notice.op.kind}${target ? ` on ${target}` : ''}: ${notice.result.outcome ?? 'refused'}`}
      </Callout>
    );
  }
  const action = (
    <Flex gap="xs">
      <Button size="sm" onClick={() => ops.resend(notice)}>Use mine</Button>
      <Button size="sm" variant="ghost" onClick={() => ops.dismiss(notice)}>Keep theirs</Button>
    </Flex>
  );
  return (
    <Callout tone="danger" action={action} details={<NoticeDiff notice={notice} />}>
      {`Conflict on ${target}: ${notice.result.by ?? 'a thread'} changed it after you looked. Nothing of yours was applied.`}
    </Callout>
  );
};

export { OpNotice };
