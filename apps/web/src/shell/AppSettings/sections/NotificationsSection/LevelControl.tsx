/* @layer renderer-app @kind component */
import { Inline, Toggle } from '@drizztdourden08/tessera';

interface LevelControlProps {
  title: string;
  toast: boolean;
  sound: boolean;
  onToast: (on: boolean) => void;
  onSound: (on: boolean) => void;
}

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
export type { LevelControlProps };
