/* @layer renderer-app @kind hook */
import { useParams } from 'react-router';
import { readRemote, REMOTE_POLL_MS, useRemoteRead } from '../../../data/remote';
import type { LocalBannerState } from '../LocalBanner.type';
import { bannerText } from './banner-text';

const useLocalBanner = (): LocalBannerState | null => {
  const { key } = useParams();
  const { data } = useRemoteRead(readRemote, REMOTE_POLL_MS);
  const text = data ? bannerText(data, key) : null;
  return data && text ? { info: data, text, boardKey: key } : null;
};

export { useLocalBanner };
