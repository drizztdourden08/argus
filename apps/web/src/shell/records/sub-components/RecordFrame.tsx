/* @layer renderer-app @kind component */
import { Box, RecordCard } from '@drizztdourden08/tessera';
import type { RecordFrameProps } from '../record-frame.type';
import { FrameLead } from './FrameLead';

const RecordFrame = (props: RecordFrameProps) => {
  const { name, kind, icon, selection, attrs, onClickCapture, ...card } = props;
  const hasLead = selection?.selecting === true || icon !== undefined || kind !== undefined;
  return (
    <Box {...attrs} onClickCapture={onClickCapture}>
      <RecordCard {...card} title={card.title ?? name} icon={hasLead ? <FrameLead kind={kind} icon={icon} selection={selection} name={name} /> : undefined} aria-label={name} />
    </Box>
  );
};

export { RecordFrame };
