/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { ShaderMount } from '@paper-design/shaders-react';
import { useMemo, useRef } from 'react';
import { GRAIN, POINTER, SIZING_UNIFORMS } from './backdrop.constants';
import { PARALLAX } from './parallax.constants';
import { toVec4 } from './to-vec4';
import { TRAIL } from './trail.constants';
import { trailToUniform } from './trail-to-uniform';
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

  usePointerDrift(motion.interactive, (frame) =>
    mount.current?.paperShaderMount?.setUniforms({
      u_pointer: [frame.point.x, frame.point.y],
      u_trail: frame.trail,
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
      u_trail: trailToUniform([]),
      u_trailDarkness: motion.interactive ? TRAIL.darkness : 0,
      u_trailPull: TRAIL.pull,
      u_trailRadius: TRAIL.radius,
      u_aspect: aspect,
    }),
    [colors, motion, aspect],
  );

  return (
    <Box className="mesh-backdrop" aria-hidden="true">
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
    </Box>
  );
};

export { MeshBackdrop };
