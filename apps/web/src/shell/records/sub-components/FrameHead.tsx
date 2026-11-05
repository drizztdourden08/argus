/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { FrameHeadProps } from '../record-frame.type';
import { FrameLead } from './FrameLead';
import { FrameSlot } from './FrameSlot';
import { FrameToggle } from './FrameToggle';

const FrameHead = (props: FrameHeadProps) => {
  const { open, onToggle, name, kind, icon, selection, code, headTitle, content, menu } = props;
  return (
    <Flex align="center" gap="sm" className="record-frame__head">
      <FrameToggle open={open} name={name} onToggle={onToggle} />
      <FrameLead kind={kind} icon={icon} selection={selection} name={name} />
      {code && <FrameSlot>{code}</FrameSlot>}
      {headTitle ? <FrameSlot grow>{headTitle}</FrameSlot> : <Flex as="span" className="record-frame__fill" />}
      {content}
      {menu}
    </Flex>
  );
};

export { FrameHead };
