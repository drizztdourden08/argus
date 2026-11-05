/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { RUN_TAGS, RUN_TIPS } from '../tests.constants';
import type { RunTagsProps } from '../tests-props.type';

const RunTags = (props: RunTagsProps) => {
  const { view } = props;
  const { stage, handed, item } = view;
  return (
    <>
      <Tag {...(RUN_TAGS[stage] ?? RUN_TAGS.draft)} fill="tint" title={RUN_TIPS[stage]} data-rstage={stage}>{stage}</Tag>
      {handed && <Tag color="neutral" fill="solid" title={`${item.id} is the run handed to you`}>handed to you</Tag>}
      {item.thread && <Tag title="The thread that owns this run">{item.thread}</Tag>}
    </>
  );
};

export { RunTags };
