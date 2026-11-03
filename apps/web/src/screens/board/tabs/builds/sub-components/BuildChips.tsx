/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { LauncherPartProps } from '../builds-props.type';
import { isMaster } from '../model';

const BuildChips = (props: LauncherPartProps) => {
  const { launcher } = props.view;
  return (
    <>
      {isMaster(launcher) && <Tag title="The main checkout: every project has one">master</Tag>}
      <Tag title={`Kind: ${launcher.kind ?? 'app'}`}>{launcher.kind ?? 'app'}</Tag>
      {launcher.current && <Tag variant="urgency" color="success" title="The valid one to start now">current</Tag>}
    </>
  );
};

export { BuildChips };
