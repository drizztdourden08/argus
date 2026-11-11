/* @layer renderer-app @kind component */
import { useReviewFeed } from '../behavior/useReviewFeed';
import type { ReviewViewProps } from '../review-props.type';
import { ReviewBody } from './ReviewBody';

const ReviewView = (props: ReviewViewProps) => {
  const { onBack, onApplied } = props;
  const feed = useReviewFeed();
  return <ReviewBody data={feed.data} loading={feed.loading} error={feed.error} onRescan={feed.rescan} onBack={onBack} onApplied={onApplied} />;
};

export { ReviewView };
