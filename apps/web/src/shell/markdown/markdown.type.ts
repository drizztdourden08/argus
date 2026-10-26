/* @layer renderer-app @kind types */
import type { MarkdownEditorProps, MarkdownImageTarget, MarkdownImages, MarkdownLinkHandler, MarkdownLinkId, MarkdownProps } from '@drizztdourden08/tessera';

type BoardMarkdownProps = Omit<MarkdownProps, 'images' | 'linkIds' | 'onLink'>;

type BoardMarkdownEditorProps = Omit<MarkdownEditorProps, 'images' | 'linkIds' | 'onLink'>;

interface MarkdownOptions {
  images: MarkdownImages;
  linkIds: MarkdownLinkId | undefined;
  onLink: MarkdownLinkHandler;
}

interface ImageLightboxProps {
  image: MarkdownImageTarget | null;
  onClose: () => void;
}

interface LightboxMarkdownProps extends BoardMarkdownProps {
  options: MarkdownOptions;
}

export type { BoardMarkdownEditorProps, BoardMarkdownProps, ImageLightboxProps, LightboxMarkdownProps, MarkdownOptions };
