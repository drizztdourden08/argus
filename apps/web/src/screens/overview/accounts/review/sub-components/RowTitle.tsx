/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import type { RowTitleProps } from '../review-tree-props.type';

const RowTitle = (props: RowTitleProps) => {
  const { row, ghost, muted, count, onToggle } = props;
  const text = (
    <Text as="span" weight={row.kind === 'folder' ? 600 : undefined} italic={ghost} tone={ghost || muted ? 'muted' : undefined} className="review-tree-title">
      {props.title}
    </Text>
  );
  return (
    <>
      {onToggle ? <Pressable className="review-tree-name" onClick={onToggle}>{text}</Pressable> : text}
      {count !== null && <Text as="span" variant="caption" tone="muted" className="review-tree-count">{count}</Text>}
    </>
  );
};

export { RowTitle };
