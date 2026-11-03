/* @layer renderer-app @kind component */
import { Markdown } from '@drizztdourden08/tessera';
import type { MarkdownImageTarget } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { withPictures } from '../behavior/with-pictures';
import type { LightboxMarkdownProps } from '../markdown.type';
import { ImageLightbox } from './ImageLightbox';

const LightboxMarkdown = ({ options, pictures, ...props }: LightboxMarkdownProps) => {
  const [image, setImage] = useState<MarkdownImageTarget | null>(null);
  return (
    <>
      <Markdown smartTimes linkTone="info" idLinkTone="neutral" {...props} {...withPictures(options, pictures)} onImage={setImage} />
      <ImageLightbox image={image} gallery={pictures?.gallery} onClose={() => setImage(null)} />
    </>
  );
};

export { LightboxMarkdown };
