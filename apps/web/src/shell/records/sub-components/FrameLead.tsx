/* @layer renderer-app @kind component */
import { Checkbox, Flex } from '@drizztdourden08/tessera';
import type { FrameLeadProps } from '../record-frame.type';
import { TypeIcon } from './TypeIcon';

const FrameLead = (props: FrameLeadProps) => {
  const { kind, icon, selection, name } = props;
  if (selection?.selecting) {
    return (
      <Flex onClickCapture={(event) => selection.onMark(event.shiftKey)}>
        <Checkbox checked={selection.selected} onChange={selection.onToggle} ariaLabel={`Select ${name}`} size="sm" />
      </Flex>
    );
  }
  return <>{icon ?? (kind ? <TypeIcon kind={kind} /> : null)}</>;
};

export { FrameLead };
