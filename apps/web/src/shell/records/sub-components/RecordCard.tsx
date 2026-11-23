/* @layer renderer-app @kind component */
import type { RecordCardProps } from '../records.type';
import { CardTitle } from './CardTitle';
import { RecordHead } from './RecordHead';

const RecordCard = (props: RecordCardProps) => {
  const { head, title, openLabel, onOpen, clamp = 3, signals, foot, attrs, onClickCapture, defaultOpen = true, content, ...look } = props;
  const parts = { title, board: head.boardKey, label: openLabel, onOpen };
  return (
    <RecordHead
      {...head}
      {...look}
      title={<CardTitle {...parts} clamp={clamp} />}
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
