/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { HeaderPopover } from '../HeaderPopover';
import { SyncStamp } from '../SyncStamp';
import { pillText } from './behavior/pill-text';
import type { LocalBannerProps } from './LocalBanner.type';

const LocalBanner = ({ state }: LocalBannerProps) => {
  const { info, text, boardKey } = state;
  return (
    <HeaderPopover trigger={pillText(info)} label="Local mode" hint={text}>
      <Text role="status">{text}</Text>
      {boardKey ? <SyncStamp sync={info.syncs?.[boardKey] ?? null} /> : null}
    </HeaderPopover>
  );
};

export { LocalBanner };
