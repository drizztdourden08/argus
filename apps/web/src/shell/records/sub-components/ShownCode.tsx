/* @layer renderer-app @kind component */
import { useShownCode } from '../behavior/useShownCode';
import type { ShownCodeProps } from '../records.type';

const ShownCode = (props: ShownCodeProps) => <>{useShownCode(props.id, props.board)}</>;

export { ShownCode };
