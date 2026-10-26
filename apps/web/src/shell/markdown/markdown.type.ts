/* @layer renderer-app @kind types */
import type { MarkdownEditorProps, MarkdownImages, MarkdownLinkHandler, MarkdownLinkId, MarkdownProps } from '@drizztdourden08/tessera';

type BoardMarkdownProps = Omit<MarkdownProps, 'images' | 'linkIds' | 'onLink'>;

type BoardMarkdownEditorProps = Omit<MarkdownEditorProps, 'images' | 'linkIds' | 'onLink'>;

interface MarkdownOptions {
  images: MarkdownImages;
  linkIds: MarkdownLinkId | undefined;
  onLink: MarkdownLinkHandler;
}

export type { BoardMarkdownEditorProps, BoardMarkdownProps, MarkdownOptions };
