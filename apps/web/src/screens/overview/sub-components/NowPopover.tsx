/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { HeaderPopover } from '../../../shell/HeaderPopover';
import type { EntriesProps } from '../boards/boards-view.type';
import { NowRail } from '../boards/NowRail';

const NowPopover = (props: EntriesProps) => (
  <HeaderPopover trigger={<Text variant="caption">Now</Text>} label="Now" hint="Running now, waiting on you and sent down">
    <NowRail entries={props.entries} />
  </HeaderPopover>
);

export { NowPopover };
