/* @layer renderer-app @kind types */
import type { MarkdownEditorProps, MarkdownImageTarget, MarkdownImages, MarkdownLinkHandler, MarkdownLinkId, MarkdownProps } from '@drizztdourden08/tessera';
import type { Pictures } from '../../data/media';

type BoardMarkdownProps = Omit<MarkdownProps, 'images' | 'linkIds' | 'onLink'> & { pictures?: Pictures };

type BoardMarkdownEditorProps = Omit<MarkdownEditorProps, 'images' | 'linkIds' | 'onLink'> & { pictures?: Pictures };

interface MarkdownOptions {
  images: MarkdownImages;
  linkIds: MarkdownLinkId | undefined;
  onLink: MarkdownLinkHandler;
}

interface ImageLightboxProps {
  image: MarkdownImageTarget | null;
  gallery?: Pictures['gallery'];
  onClose: () => void;
}

interface LightboxMarkdownProps extends BoardMarkdownProps {
  options: MarkdownOptions;
}

interface StandaloneEditorProps extends BoardMarkdownEditorProps {
  options: MarkdownOptions;
}

export type { BoardMarkdownEditorProps, BoardMarkdownProps, ImageLightboxProps, LightboxMarkdownProps, MarkdownOptions, StandaloneEditorProps };
