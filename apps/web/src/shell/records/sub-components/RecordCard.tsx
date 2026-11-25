/* @layer renderer-app @kind component */
import type { RecordCardProps } from '../records.type';
import { CardTitle } from './CardTitle';
import { RecordHead } from './RecordHead';

const RecordCard = (props: RecordCardProps) => {
  const { head, title, openLabel, onOpen, clamp = 1, time, compact, signals, foot, keepFoot = false, attrs, onClickCapture, defaultOpen = true, content, ...look } = props;
  const parts = { title, board: head.boardKey, label: openLabel, onOpen };
  return (
    <RecordHead
      {...head}
      {...look}
      density={compact ? 'compact' : undefined}
      time={time ?? head.time}
      title={<CardTitle {...parts} clamp={clamp} />}
      defaultOpen={defaultOpen}
      content={compact ? <>{content}{signals}</> : content}
      attrs={attrs}
      onClickCapture={onClickCapture}
    >
      {compact ? undefined : signals}
      {compact && !keepFoot ? undefined : foot}
    </RecordHead>
  );
};

export { RecordCard };
