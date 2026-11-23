/* @layer renderer-app @kind logic */
import { TAIL_MAX } from '../OverlordChat.constants';

const tailOf = (text: string): string => {
  const flat = text.replace(/\s+/g, ' ').trim();
  return flat.length > TAIL_MAX ? `...${flat.slice(flat.length - TAIL_MAX)}` : flat;
};

export { tailOf };
