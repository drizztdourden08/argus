/* @layer renderer-app @kind component */
import { Checkbox, Flex } from '@drizztdourden08/tessera';
import type { FrameLeadProps } from '../record-frame.type';
import { FrameSlot } from './FrameSlot';
import { TypeIcon } from './TypeIcon';

const FrameLead = (props: FrameLeadProps) => {
  const { kind, icon, selection, name } = props;
  if (selection?.selecting) {
    return (
      <FrameSlot>
        <Flex onClickCapture={(event) => selection.onMark(event.shiftKey)}>
          <Checkbox checked={selection.selected} onChange={selection.onToggle} ariaLabel={`Select ${name}`} size="sm" />
        </Flex>
      </FrameSlot>
    );
  }
  const mark = icon ?? (kind ? <TypeIcon kind={kind} /> : null);
  return mark ? <FrameSlot>{mark}</FrameSlot> : null;
};

export { FrameLead };
