/* @layer renderer-app @kind component */
import { Inline, Toggle } from '@drizztdourden08/tessera';
import type { LevelControlProps } from './LevelControl.type';

const LevelControl = (props: LevelControlProps) => {
  const { title, toast, sound, onToast, onSound } = props;
  return (
    <Inline gap="md">
      <Toggle label="Toast" aria-label={`${title} toast`} checked={toast} onChange={onToast} />
      <Toggle label="Sound" aria-label={`${title} sound`} checked={sound} onChange={onSound} />
    </Inline>
  );
};

export { LevelControl };
