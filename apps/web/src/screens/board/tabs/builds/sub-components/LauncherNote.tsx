/* @layer renderer-app @kind component */
import { Disclosure, Markdown } from '@drizztdourden08/tessera';
import type { LauncherPartProps } from '../builds-props.type';

const LauncherNote = (props: LauncherPartProps) => {
  const { launcher } = props.view;
  const text = launcher.retiredAt ? launcher.retireNote ?? launcher.note : launcher.note;
  if (!text) return null;
  return <Disclosure size="sm" summary="Note"><Markdown size="sm">{text}</Markdown></Disclosure>;
};

export { LauncherNote };
