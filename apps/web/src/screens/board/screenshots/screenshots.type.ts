/* @layer renderer-app @kind types */
import type { DropZoneStatus } from '@drizztdourden08/tessera';

interface Screenshot {
  n: number;
  url: string;
  caption: string;
}

interface ScreenshotGalleryProps {
  shots: readonly Screenshot[];
}

interface ScreenshotAttachProps {
  label: string;
  caption: string;
  onCaptionChange: (caption: string) => void;
  uploading: boolean;
  status: DropZoneStatus | undefined;
  onDrop: (files: File[]) => void;
}

export type { ScreenshotAttachProps, ScreenshotGalleryProps };
