/* @layer renderer-app @kind component */
import { DateText } from '@drizztdourden08/tessera';
import type { ShortAgeProps } from '../overview.type';

const ShortAge = (props: ShortAgeProps) => {
  const { value, ago = false } = props;
  return (
    <>
      <DateText value={value} format="elapsed" length="short" />
      {ago ? ' ago' : null}
    </>
  );
};

export { ShortAge };
