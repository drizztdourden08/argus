/* @layer renderer-app @kind component */
import { Icon, Spinner, Status } from '@drizztdourden08/tessera';
import type { ChipMarkProps } from '../RegisteredStrip.type';

const ChipMark = (props: ChipMarkProps) => {
  const { look, name } = props;
  if (look.mark === 'spin') {
    return (
      <>
        <Spinner size="sm" label="Wake on its way" />
        <Status tone={look.tone}>{name}</Status>
      </>
    );
  }
  if (look.mark === 'warn') {
    return (
      <Status tone={look.tone}>
        <Icon name="triangle-alert" size={12} aria-hidden />
        {name}
      </Status>
    );
  }
  return <Status tone={look.tone} dot>{name}</Status>;
};

export { ChipMark };
