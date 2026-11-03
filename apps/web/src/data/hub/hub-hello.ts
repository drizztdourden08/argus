/* @layer renderer-app @kind logic */
import { MUX_VERSION } from './hub.constants';
import type { HubMessage } from './hub.type';

const helloOf = (epoch: number): HubMessage => ({ type: 'hub-hello', epoch, v: MUX_VERSION });

export { helloOf };
