/* @layer renderer-app @kind hook */
import { useState } from 'react';
import type { FrameOpenState } from '../record-frame.type';

const useFrameOpen = (props: FrameOpenState): [boolean, () => void] => {
  const { open, defaultOpen = false, onOpenChange } = props;
  const [inner, setInner] = useState(defaultOpen);
  const value = open ?? inner;
  const toggle = () => {
    setInner(!value);
    onOpenChange?.(!value);
  };
  return [value, toggle];
};

export { useFrameOpen };
