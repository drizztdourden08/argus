/* @layer renderer-app @kind component */
import { Button, Callout, CopyValue, Flex } from '@drizztdourden08/tessera';
import type { NoticeBannerProps } from '../worktrees-props.type';

const NoticeBanner = (props: NoticeBannerProps) => {
  const { notice, onDismiss } = props;
  const copies = notice.backup !== null || notice.restore !== null;
  return (
    <Callout
      tone="success"
      action={<Button size="xs" variant="ghost" onClick={onDismiss}>Dismiss</Button>}
      details={copies ? (
        <Flex direction="column" gap="2xs">
          {notice.backup && <CopyValue value={notice.backup} label="backup path" mono truncate="middle" />}
          {notice.restore && <CopyValue value={notice.restore} label="restore command" mono truncate="middle" />}
        </Flex>
      ) : undefined}
    >
      {notice.text}
    </Callout>
  );
};

export { NoticeBanner };
