/* @layer renderer-app @kind types */
import type { BrandApp } from '@drizztdourden08/tessera/brand';

interface BrandMarkPickerProps {
  value: BrandApp | null;
  onPick: (app: BrandApp) => void;
  label: string;
  disabled?: boolean;
}

export type { BrandMarkPickerProps };
