/* @layer renderer-app @kind component */
import { Disclosure } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import type { LauncherPartProps } from '../builds-props.type';

const LauncherNote = (props: LauncherPartProps) => {
  const { launcher } = props.view;
  const text = launcher.retiredAt ? launcher.retireNote ?? launcher.note : launcher.note;
  if (!text) return null;
  return <Disclosure size="sm" summary="Note"><BoardMarkdown size="sm">{text}</BoardMarkdown></Disclosure>;
};

export { LauncherNote };
