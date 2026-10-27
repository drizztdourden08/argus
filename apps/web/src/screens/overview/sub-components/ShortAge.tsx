/* @layer renderer-app @kind component */
import { formatAgo } from '../../../format';
import { useNow } from '../clock';
import type { ShortAgeProps } from '../overview.type';

const ShortAge = (props: ShortAgeProps) => {
  const { value, ago = false } = props;
  const now = useNow();
  return (
    <>
      {formatAgo(value, now)}
      {ago ? ' ago' : null}
    </>
  );
};

export { ShortAge };
