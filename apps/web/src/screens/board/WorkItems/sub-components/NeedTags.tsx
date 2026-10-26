/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { NeedTagsProps } from '../WorkItems.type';

const NeedTags = (props: NeedTagsProps) => {
  const { need } = props;
  const yours = need.questions + need.steps + need.tests;
  const detail = `${need.questions} open, ${need.steps} manual, ${need.tests} to run`;
  return (
    <>
      {yours > 0 ? <Tag variant="urgency" color="warning" title={detail}>needs you</Tag> : null}
      {need.decisions > 0 ? <Tag>{need.decisions} to pick up</Tag> : null}
    </>
  );
};

export { NeedTags };
