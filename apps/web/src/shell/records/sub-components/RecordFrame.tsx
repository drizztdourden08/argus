/* @layer renderer-app @kind component */
import { Box, RecordCard } from '@drizztdourden08/tessera';
import type { RecordFrameProps } from '../record-frame.type';
import { FrameLead } from './FrameLead';

const RecordFrame = (props: RecordFrameProps) => {
  const { name, label, kind, icon, selection, attrs, onClickCapture, onKeyDown, ...card } = props;
  const hasLead = selection?.selecting === true || icon !== undefined || kind !== undefined;
  return (
    <Box {...attrs} onClickCapture={onClickCapture} onKeyDown={onKeyDown}>
      <RecordCard {...card} title={card.title ?? name} icon={hasLead ? <FrameLead kind={kind} icon={icon} selection={selection} name={name} /> : undefined} name={name} aria-label={label ?? name} />
    </Box>
  );
};

export { RecordFrame };
