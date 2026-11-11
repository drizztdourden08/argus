/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { EMPTY_REVIEW, NOTHING_TEXT } from '../review.constants';
import type { ReviewEmptyProps } from '../review-props.type';

const ReviewEmpty = (props: ReviewEmptyProps) => {
  const { loaded, total, shown } = props;
  if (!loaded || shown > 0) return null;
  return <EmptyState message={total === 0 ? EMPTY_REVIEW : NOTHING_TEXT} />;
};

export { ReviewEmpty };
