#version 450

layout (location = 0) out vec3 interp_colour;

layout (push_constant) uniform _matrices {
	mat4 mvp;
} matrices;

vec3 pos[3] = vec3[](
	vec3(0., -.5, 0.),
	vec3(-.5, .5, 0.),
	vec3(.5, .5, 0.)
);

vec3 colours[3] = vec3[](
	vec3(1., 0., 0.),
	vec3(0., 1., 0.),
	vec3(0., 0., 1.)
);

void main() {
	gl_Position = matrices.mvp * vec4(pos[gl_VertexIndex], 1.);
	interp_colour = colours[gl_VertexIndex];
}
