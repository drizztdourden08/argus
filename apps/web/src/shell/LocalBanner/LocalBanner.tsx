/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { useParams } from 'react-router';
import { readRemote, REMOTE_POLL_MS, useRemoteRead } from '../../data/remote';
import { bannerText } from './behavior/banner-text';

const LocalBanner = () => {
  const { key } = useParams();
  const { data } = useRemoteRead(readRemote, REMOTE_POLL_MS);
  const text = data ? bannerText(data, key) : null;
  if (!text) return null;
  return <Text variant="caption" tone="muted" role="status">{text}</Text>;
};

export { LocalBanner };
