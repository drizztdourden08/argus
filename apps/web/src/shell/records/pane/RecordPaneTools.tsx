/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { DrawerToolbar } from '../../../screens/board/shared';
import { paneLink } from './model/pane-link';
import type { RecordPaneToolsProps } from './pane.type';

const RecordPaneTools = (props: RecordPaneToolsProps) => {
  const { code, pane, previous, next, wide, onWide, children } = props;
  const back = pane.canBack ? (
    <IconButton label="Back" size="xs" variant="ghost" onClick={pane.back}>
      <Icon name="arrow-left" />
    </IconButton>
  ) : null;
  return (
    <DrawerToolbar id={code} previous={previous} next={next} label="Record tools" wide={wide} onWide={onWide} onStep={pane.step} link={paneLink} before={back}>
      {children}
    </DrawerToolbar>
  );
};

export { RecordPaneTools };
