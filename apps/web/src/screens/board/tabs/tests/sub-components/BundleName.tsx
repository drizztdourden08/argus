/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { NO_BUNDLE } from '../model';
import type { BundleNameProps } from '../tests-props.type';

const BundleName = (props: BundleNameProps) => {
  const { tag, title, muted = false } = props;
  const name = tag === NO_BUNDLE ? 'No bundle' : tag;
  return (
    <>
      <Text font="title" tone={muted ? 'muted' : undefined} className="bundle__tag">{name}</Text>
      {title && <Text smartTimes tone={muted ? 'muted' : 'dim'} className="bundle__title">{title}</Text>}
    </>
  );
};

export { BundleName };
