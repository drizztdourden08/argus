/* @layer renderer-app @kind component */
import { MarkdownEditor } from '@drizztdourden08/tessera';
import { withPictures } from '../behavior/with-pictures';
import type { StandaloneEditorProps } from '../markdown.type';

const StandaloneEditor = ({ options, pictures, ...props }: StandaloneEditorProps) => <MarkdownEditor {...props} {...withPictures(options, pictures)} />;

export { StandaloneEditor };
