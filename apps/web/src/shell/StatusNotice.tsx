/* @layer renderer-app @kind component */
import { Callout } from '@drizztdourden08/tessera';
import { NOTICES } from './shell.constants';
import type { StatusNoticeProps } from './shell.type';

const StatusNotice = (props: StatusNoticeProps) => {
  const { status } = props;
  if (status !== 'unavailable' && status !== 'unauthorized') return null;
  const notice = NOTICES[status];
  return <Callout tone={notice.tone}>{notice.text}</Callout>;
};

export { StatusNotice };
