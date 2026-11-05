/* @layer renderer-app @kind constants */
import { SHADER_NOISE } from './shader-noise.constants';
import { SHADER_TRAIL } from './shader-trail.constants';

const MAX_COLORS = 10;

const MESH_SHADER = `#version 300 es
precision mediump float;

uniform float u_time;

uniform vec4 u_colors[${MAX_COLORS}];
uniform float u_colorsCount;

uniform float u_distortion;
uniform float u_swirl;
uniform float u_grainMixer;
uniform float u_grainOverlay;

uniform vec2 u_pointer;
uniform vec4 u_parallax;
uniform float u_aspect;

in vec2 v_objectUV;
out vec4 fragColor;
${SHADER_NOISE}${SHADER_TRAIL}
vec2 getPosition(int i, float t) {
  float a = float(i) * .37;
  float b = .6 + fract(float(i) / 3.) * .9;
  float c = .8 + fract(float(i + 1) / 4.);

  float x = sin(t * b + a);
  float y = cos(t * c + a * 1.5);

  return .5 + .5 * vec2(x, y);
}

float getDepth(int i) {
  return mix(u_parallax.y, u_parallax.x, fract(float(i) * .618 + .13));
}

void main() {
  vec2 uv = v_objectUV;
  uv += .5;
  vec2 shift = vec2(u_pointer.x, 1. - u_pointer.y) - .5;
  vec3 trail = trailInk(uv);
  vec2 grainUV = (uv - shift * u_parallax.z) * vec2(1000. * u_aspect, 1000.);

  float mixerGrain = 0.;
  if (u_grainMixer > 0.) {
    mixerGrain = .4 * u_grainMixer * (valueNoise(grainUV) - .5);
  }

  const float firstFrameOffset = 41.5;
  float t = .5 * (u_time + firstFrameOffset);

  vec2 swirlCenter = vec2(.5) - shift * u_parallax.w;
  float radius = smoothstep(0., 1., length(uv - swirlCenter));
  float center = 1. - radius;
  for (float i = 1.; i <= 2.; i++) {
    uv.x += u_distortion * center / i * sin(t + i * .4 * smoothstep(.0, 1., uv.y)) * cos(.2 * t + i * 2.4 * smoothstep(.0, 1., uv.y));
    uv.y += u_distortion * center / i * cos(t + i * 2. * smoothstep(.0, 1., uv.x));
  }

  vec2 uvRotated = uv - swirlCenter;
  float angle = 3. * u_swirl * radius;
  uvRotated = rotate(uvRotated, -angle) + swirlCenter;
  uvRotated -= trail.yz * trail.x * u_trailPull;

  vec3 color = vec3(0.);
  float opacity = 0.;
  float totalWeight = 0.;

  for (int i = 0; i < ${MAX_COLORS}; i++) {
    if (i >= int(u_colorsCount)) break;

    vec2 pos = getPosition(i, t) - shift * getDepth(i) + mixerGrain;
    vec3 colorFraction = u_colors[i].rgb * u_colors[i].a;
    float opacityFraction = u_colors[i].a;

    float dist = length(uvRotated - pos);

    dist = pow(dist, 3.5);
    float weight = 1. / (dist + 1e-3);
    color += colorFraction * weight;
    opacity += opacityFraction * weight;
    totalWeight += weight;
  }

  color /= max(1e-4, totalWeight);
  opacity /= max(1e-4, totalWeight);
  color *= 1. - trail.x * u_trailDarkness;

  if (u_grainOverlay > 0.) {
    float grainOverlay = valueNoise(rotate(grainUV, 1.) + vec2(3.));
    grainOverlay = mix(grainOverlay, valueNoise(rotate(grainUV, 2.) + vec2(-1.)), .5);
    grainOverlay = pow(grainOverlay, 1.3);

    float grainOverlayV = grainOverlay * 2. - 1.;
    vec3 grainOverlayColor = vec3(step(0., grainOverlayV));
    float grainOverlayStrength = u_grainOverlay * abs(grainOverlayV);
    grainOverlayStrength = pow(grainOverlayStrength, .8);
    color = mix(color, grainOverlayColor, .35 * grainOverlayStrength);

    opacity += .5 * grainOverlayStrength;
  }
  opacity = clamp(opacity, 0., 1.);

  fragColor = vec4(color, opacity);
}
`;

export { MAX_COLORS, MESH_SHADER };
