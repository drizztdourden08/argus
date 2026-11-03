/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { RowChipTagProps } from '../records.type';
import { RecordLink } from './RecordLink';
import { ShownCode } from './ShownCode';

const RowChipTag = (props: RowChipTagProps) => {
  const { chip, board } = props;
  const tag = <Tag title={chip.title}>{chip.code ? <ShownCode id={chip.text} board={board} /> : chip.text}</Tag>;
  return chip.link ? <RecordLink id={chip.link} board={board}>{tag}</RecordLink> : tag;
};

export { RowChipTag };
