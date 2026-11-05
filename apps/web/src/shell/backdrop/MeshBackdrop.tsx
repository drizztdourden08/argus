/* @layer renderer-app @kind component */
import { MeshGradient } from '@paper-design/shaders-react';
import { useBackdropColors } from './useBackdropColors';
import { useBackdropMotion } from './useBackdropMotion';
import './MeshBackdrop.css';

const MeshBackdrop = () => {
  const colors = useBackdropColors();
  const motion = useBackdropMotion();
  return (
    <div className="mesh-backdrop" aria-hidden="true">
      <MeshGradient
        colors={[...colors]}
        speed={motion.speed}
        distortion={motion.distortion}
        swirl={motion.swirl}
        width="100%"
        height="100%"
      />
    </div>
  );
};

export { MeshBackdrop };
