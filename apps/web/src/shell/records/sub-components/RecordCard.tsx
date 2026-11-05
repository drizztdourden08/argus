/* @layer renderer-app @kind component */
import type { RecordCardProps } from '../records.type';
import { CardTitle } from './CardTitle';
import { RecordHead } from './RecordHead';

const RecordCard = (props: RecordCardProps) => {
  const { head, title, openLabel, onOpen, signals, foot, attrs, onClickCapture, defaultOpen = true, content } = props;
  const parts = { title, board: head.boardKey, label: openLabel, onOpen };
  return (
    <RecordHead
      {...head}
      title={<CardTitle {...parts} clamp={3} />}
      defaultOpen={defaultOpen}
      content={content}
      attrs={attrs}
      onClickCapture={onClickCapture}
    >
      {signals}
      {foot}
    </RecordHead>
  );
};

export { RecordCard };
