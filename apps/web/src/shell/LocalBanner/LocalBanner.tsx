/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { useParams } from 'react-router';
import { readRemote, REMOTE_POLL_MS, useRemoteRead } from '../../data/remote';
import { HeaderPopover } from '../HeaderPopover';
import { SyncStamp } from '../SyncStamp';
import { bannerText } from './behavior/banner-text';
import { pillText } from './behavior/pill-text';

const LocalBanner = () => {
  const { key } = useParams();
  const { data } = useRemoteRead(readRemote, REMOTE_POLL_MS);
  const text = data ? bannerText(data, key) : null;
  if (!data || !text) return null;
  return (
    <HeaderPopover trigger={pillText(data)} label="Local mode" hint={text}>
      <Text role="status">{text}</Text>
      {key ? <SyncStamp sync={data.syncs?.[key] ?? null} /> : null}
    </HeaderPopover>
  );
};

export { LocalBanner };
