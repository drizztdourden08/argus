/* @layer renderer-app @kind component */
import type { RecordCardProps } from '../records.type';
import { CardTitle } from './CardTitle';
import { RecordHead } from './RecordHead';

const RecordCard = (props: RecordCardProps) => {
  const { head, title, openLabel, onOpen, clamp = 1, time, compact, signals, foot, attrs, onClickCapture, defaultOpen = true, content, ...look } = props;
  const parts = { title, board: head.boardKey, label: openLabel, onOpen };
  return (
    <RecordHead
      {...head}
      {...look}
      time={time ?? head.time}
      title={<CardTitle {...parts} clamp={clamp} />}
      defaultOpen={defaultOpen}
      content={content}
      attrs={compact ? { ...attrs, 'data-compact': 'true' } : attrs}
      onClickCapture={onClickCapture}
    >
      {signals}
      {foot}
    </RecordHead>
  );
};

export { RecordCard };
