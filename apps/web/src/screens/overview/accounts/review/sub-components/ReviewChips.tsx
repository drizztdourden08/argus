/* @layer renderer-app @kind component */
import { Inline, Tag } from '@drizztdourden08/tessera';
import type { ReviewChipsProps } from '../review-props.type';

const ReviewChips = (props: ReviewChipsProps) => {
  const { accounts, counts, edits } = props;
  return (
    <Inline gap="sm" align="center" wrap>
      {accounts.map((account) => <Tag key={account.key}>{`${account.label ?? account.key} ${counts?.chats?.[account.key] ?? 0}`}</Tag>)}
      <Tag variant="urgency" color="warning" fill="tint">{`${counts?.differences ?? 0} differences`}</Tag>
      {edits > 0 && <Tag variant="urgency" color="info" fill="tint">{`${edits} edits pending`}</Tag>}
    </Inline>
  );
};

export { ReviewChips };
