/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { ShaderMount } from '@paper-design/shaders-react';
import { useMemo, useRef, useState } from 'react';
import { GRAIN, POINTER, SIZING_UNIFORMS } from './backdrop.constants';
import { BLOB } from './blob.constants';
import { hasWebgl2 } from './has-webgl2';
import { PARALLAX } from './parallax.constants';
import { toVec4 } from './to-vec4';
import { MAX_COLORS, MESH_SHADER } from './mesh-shader.constants';
import type { Mount } from './MeshBackdrop.type';
import { useBackdropColors } from './useBackdropColors';
import { useBackdropMotion } from './useBackdropMotion';
import { usePointerDrift } from './usePointerDrift';
import { useViewportAspect } from './useViewportAspect';
import './MeshBackdrop.css';

const MeshBackdrop = () => {
  const colors = useBackdropColors();
  const motion = useBackdropMotion();
  const aspect = useViewportAspect();
  const mount = useRef<Mount>(null);
  const [webgl] = useState(hasWebgl2);

  usePointerDrift(motion.interactive, (frame) =>
    mount.current?.paperShaderMount?.setUniforms({
      u_pointer: [frame.point.x, frame.point.y],
      u_blob: [frame.blob.x, 1 - frame.blob.y],
      u_blobWeight: frame.blobWeight,
    }),
  );

  const uniforms = useMemo(
    () => ({
      ...SIZING_UNIFORMS,
      u_colors: colors.slice(0, MAX_COLORS).map(toVec4),
      u_colorsCount: Math.min(colors.length, MAX_COLORS),
      u_distortion: motion.distortion,
      u_swirl: motion.swirl,
      u_grainMixer: GRAIN.mixer,
      u_grainOverlay: GRAIN.overlay,
      u_pointer: [POINTER.center, POINTER.center],
      u_parallax: motion.interactive ? [PARALLAX.near, PARALLAX.far, PARALLAX.grain, PARALLAX.swirl] : [0, 0, 0, 0],
      u_blob: [POINTER.center, POINTER.center],
      u_blobWeight: 0,
      u_blobColor: [...BLOB.color],
      u_aspect: aspect,
    }),
    [colors, motion, aspect],
  );

  return (
    <Box className="mesh-backdrop" aria-hidden="true">
      {webgl && (
        <ShaderMount
          ref={(element) => {
            mount.current = element;
          }}
          fragmentShader={MESH_SHADER}
          uniforms={uniforms}
          speed={motion.speed}
          width="100%"
          height="100%"
        />
      )}
    </Box>
  );
};

export { MeshBackdrop };
