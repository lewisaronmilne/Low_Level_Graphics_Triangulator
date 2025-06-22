#version 450

layout(origin_upper_left) in vec4 gl_FragCoord;

layout(binding = 0) uniform b0 { uvec2 dims; vec2 clr_rg, clr_ba; };

struct Mask { float left; float top; float width; float height; };
layout(binding = 1) buffer b1 { Mask[] masks; };

layout(location = 0) in vec4 v_colour;
layout(location = 0) out vec4 out_colour;

void main() 
{
    vec2 uv = 2*(gl_FragCoord.xy / dims) - 1.0;
    vec4 clr_rgba = vec4(clr_rg, clr_ba);

    Mask m = masks[0];
    if ((uv.x > m.left  && uv.x < (m.left + m.width)) && (uv.y > m.top && uv.y < (m.top + m.height)))
        { out_colour = vec4((uv.x+1)/2 * (uv.y+1)/2, (-1*uv.x+1)/2 * (uv.y+1)/2, (uv.x+1)/2 * (-1*uv.y+1)/2, 1.0); }
    else
        { out_colour = clr_rgba; }
}