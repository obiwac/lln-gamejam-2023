#version 450

layout (location = 0) out vec3 interp_colour;

vec2 pos[3] = vec2[](
	vec2(0., -.5),
	vec2(-.5, .5),
	vec2(.5, .5)
);

vec3 colours[3] = vec3[](
	vec3(1., 0., 0.),
	vec3(0., 1., 0.),
	vec3(0., 0., 1.)
);

void main() {
	gl_Position = vec4(pos[gl_VertexIndex], 0., 1.);
	interp_colour = colours[gl_VertexIndex];
}
