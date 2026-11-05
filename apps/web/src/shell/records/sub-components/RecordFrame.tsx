/* @layer renderer-app @kind component */
import { Box, Grid, RecordCard } from '@drizztdourden08/tessera';
import type { RecordFrameProps } from '../record-frame.type';
import { activateOnBody } from '../behavior/activate-on-body';
import { FrameLead } from './FrameLead';

const RecordFrame = (props: RecordFrameProps) => {
  const { name, label, kind, icon, selection, attrs, onClickCapture, onKeyDown, onActivate, ...card } = props;
  const hasLead = selection?.selecting === true || icon !== undefined || kind !== undefined;
  const Wrap = card.fill === true ? Grid : Box;
  return (
    <Wrap {...attrs} onClickCapture={onClickCapture} onKeyDown={onKeyDown} onClick={onActivate && activateOnBody(onActivate)}>
      <RecordCard {...card} title={card.title ?? name} icon={hasLead ? <FrameLead kind={kind} icon={icon} selection={selection} name={name} /> : undefined} name={name} aria-label={label ?? name} />
    </Wrap>
  );
};

export { RecordFrame };
