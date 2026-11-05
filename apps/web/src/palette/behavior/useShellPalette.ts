/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import type { Palette } from '../palette.type';

const useShellPalette = (palette: Palette | undefined): void => {
  useEffect(() => {
    const root = document.documentElement;
    if (palette) root.dataset.palette = palette;
    else delete root.dataset.palette;
    return () => {
      delete root.dataset.palette;
    };
  }, [palette]);
};

export { useShellPalette };
