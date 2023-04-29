#version 450

layout (location = 0) in vec2 interp_uv;

layout (location = 0) out vec4 colour;

layout(binding = 1) uniform sampler2D texSampler;

void main() { 
	// colour = vec4(texture(texSampler,interp_uv).xyz, 1.);
	colour = texture(texSampler,interp_uv);

	if (colour.a < 0.5)
		discard;
}
