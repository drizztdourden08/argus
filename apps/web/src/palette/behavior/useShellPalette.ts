/* @layer renderer-app @kind logic */
import { useSwitchPalette } from '@drizztdourden08/tessera';
import { useEffect } from 'react';
import type { Palette } from '../palette.type';

const useShellPalette = (palette: Palette | undefined): void => {
  const switchPalette = useSwitchPalette();
  useEffect(() => {
    const root = document.documentElement;
    switchPalette(() => {
      if (palette) root.dataset.palette = palette;
      else delete root.dataset.palette;
    });
  }, [palette, switchPalette]);
  useEffect(
    () => () => {
      delete document.documentElement.dataset.palette;
    },
    [],
  );
};

export { useShellPalette };
