#type vertex
#version 330 core

uniform vec2 uScreenSize;

layout (location=0) in vec2 aPos;
layout (location=1) in vec2 aTexCoords;
layout (location=2) in vec4 aColor;

out vec2 fTexCoords;
out vec4 fColor;

// 0-1 linear  from  0-255 sRGB
vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));
}

// 0-1 linear  from  0-255 sRGBA
vec4 linear_from_srgba(vec4 srgba) {
    return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

// 0-255 sRGB  from  0-1 linear
vec3 srgb_from_linear(vec3 rgb) {
    bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
    vec3 lower = rgb * vec3(3294.6);
    vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
    return mix(higher, lower, vec3(cutoff));
}

// 0-255 sRGBA  from  0-1 linear
vec4 srgba_from_linear(vec4 rgba) {
    return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
}

vec4 srgbToLinear(vec4 srgba) {
    vec3 linearRGB;
    for(int i = 0; i < 3; ++i) {
        if (srgba[i] <= 0.04045) {
            linearRGB[i] = srgba[i] / 12.92;
        } else {
            linearRGB[i] = pow((srgba[i] + 0.055) / 1.055, 2.4);
        }
    }
    return vec4(linearRGB, srgba.a); // RGB转换，Alpha保持不变
}

void main() {
    gl_Position = vec4(
    2.0 * aPos.x / uScreenSize.x - 1.0,
    1.0 - 2.0 * aPos.y / uScreenSize.y,
    0.0,
    1.0
    );
    fTexCoords = aTexCoords;
    fColor = linear_from_srgba(aColor);
    fColor.a = pow(fColor.a, 1.6);
}

#type fragment
#version 330 core

uniform sampler2D uSampler;

in vec2 fTexCoords;
in vec4 fColor;

// 0-1 linear  from  0-255 sRGB
vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));
}

// 0-1 linear  from  0-255 sRGBA
vec4 linear_from_srgba(vec4 srgba) {
    return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

// 0-255 sRGB  from  0-1 linear
vec3 srgb_from_linear(vec3 rgb) {
    bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
    vec3 lower = rgb * vec3(3294.6);
    vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
    return mix(higher, lower, vec3(cutoff));
}

// 0-255 sRGBA  from  0-1 linear
vec4 srgba_from_linear(vec4 rgba) {
    return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
}

void main() {
    // We must decode the colors, since WebGL1 doesn't come with sRGBA textures:
    vec4 texture_rgba = linear_from_srgba(texture2D(uSampler, fTexCoords) * 255.0);
    // WebGL1 doesn't support linear blending in the framebuffer,
    // so we do a hack here where we change the premultiplied alpha
    // to do the multiplication in gamma space instead:
    // Unmultiply alpha:
    if (texture_rgba.a > 0.0) {
        texture_rgba.rgb /= texture_rgba.a;
    }
    // Empiric tweak to make e.g. shadows look more like they should:
    texture_rgba.a *= sqrt(texture_rgba.a);
    // To gamma:
    texture_rgba = srgba_from_linear(texture_rgba) / 255.0;
    // Premultiply alpha, this time in gamma space:
    if (texture_rgba.a > 0.0) {
        texture_rgba.rgb *= texture_rgba.a;
    }
    /// Multiply vertex color with texture color (in linear space).
    gl_FragColor = fColor * texture_rgba;
}