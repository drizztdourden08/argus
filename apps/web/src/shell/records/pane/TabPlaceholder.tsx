/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { TabPlaceholderProps } from './pane.type';

const TabPlaceholder = (props: TabPlaceholderProps) => {
  const { tab, label, data } = props;
  return (
    <Text tone="muted" italic data-placeholder={tab}>
      {`The ${label} tab of ${data.code} is not shown yet.`}
    </Text>
  );
};

export { TabPlaceholder };
