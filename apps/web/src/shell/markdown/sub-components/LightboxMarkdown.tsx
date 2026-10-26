/* @layer renderer-app @kind component */
import { Markdown } from '@drizztdourden08/tessera';
import type { MarkdownImageTarget } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { LightboxMarkdownProps } from '../markdown.type';
import { ImageLightbox } from './ImageLightbox';

const LightboxMarkdown = ({ options, ...props }: LightboxMarkdownProps) => {
  const [image, setImage] = useState<MarkdownImageTarget | null>(null);
  return (
    <>
      <Markdown {...props} {...options} onImage={setImage} />
      <ImageLightbox image={image} onClose={() => setImage(null)} />
    </>
  );
};

export { LightboxMarkdown };
