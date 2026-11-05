/* @layer renderer-app @kind constants */
import { TRAIL } from './trail.constants';

const SHADER_TRAIL = `
uniform vec3 u_trail[${TRAIL.size}];
uniform float u_trailDarkness;
uniform float u_trailPull;
uniform float u_trailRadius;

vec3 trailInk(vec2 uv) {
  vec2 scale = vec2(u_aspect, 1.);
  float ink = 0.;
  vec2 away = vec2(0.);
  for (int i = 0; i < ${TRAIL.size}; i++) {
    vec3 a = u_trail[i];
    if (a.z <= 0.) break;
    vec3 b = a;
    if (i + 1 < ${TRAIL.size} && u_trail[i + 1].z > 0.) b = u_trail[i + 1];
    vec2 ab = (b.xy - a.xy) * scale;
    vec2 ap = (uv - a.xy) * scale;
    float h = clamp(dot(ap, ab) / max(dot(ab, ab), 1e-6), 0., 1.);
    vec2 off = ap - ab * h;
    float weight = mix(a.z, b.z, h);
    float reach = u_trailRadius * (.3 + .7 * weight);
    float dist = length(off);
    float k = weight * (1. - smoothstep(0., reach, dist));
    if (k > ink) {
      ink = k;
      away = off / max(dist, 1e-4) / scale;
    }
  }
  return vec3(ink * ink * (3. - 2. * ink), away);
}
`;

export { SHADER_TRAIL };
