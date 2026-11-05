/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useFrameOpen } from '../behavior/useFrameOpen';
import type { RecordFrameProps } from '../record-frame.type';
import { FrameLead } from './FrameLead';
import { FrameSlot } from './FrameSlot';
import { FrameToggle } from './FrameToggle';
import './RecordFrame.css';

const RecordFrame = (props: RecordFrameProps) => {
  const { name, kind, icon, selection, code, title, compactTitle, pinTitle = false, content, menu, density = 'card', attrs, onClickCapture, children } = props;
  const [open, toggle] = useFrameOpen(props);
  const inHead = pinTitle || !open;
  const headTitle = pinTitle ? title : (compactTitle ?? title);
  return (
    <Flex direction="column" className={`record-frame record-frame--${density}`} {...attrs} onClickCapture={onClickCapture}>
      <Flex align="center" gap="sm" className="record-frame__head">
        <FrameToggle open={open} name={name} onToggle={toggle} />
        <FrameLead kind={kind} icon={icon} selection={selection} name={name} />
        {code && <FrameSlot>{code}</FrameSlot>}
        {inHead && headTitle && <FrameSlot grow>{headTitle}</FrameSlot>}
        {!inHead && <span className="record-frame__fill" />}
        {content}
        {menu}
      </Flex>
      {open && (
        <Flex direction="column" gap="xs" className="record-frame__body">
          {!pinTitle && title}
          {children}
        </Flex>
      )}
    </Flex>
  );
};

export { RecordFrame };
