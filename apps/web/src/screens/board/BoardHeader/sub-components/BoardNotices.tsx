/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { NOTICES_HINT } from '../BoardHeader.constants';
import type { BoardNoticesProps } from '../BoardHeader.type';
import { NoticeList } from './NoticeList';
import './BoardNotices.css';

const BoardNotices = (props: BoardNoticesProps) => (
  <>
    <Text variant="caption" tone="muted">{NOTICES_HINT}</Text>
    <NoticeList notices={props.notices} />
  </>
);

export { BoardNotices };
