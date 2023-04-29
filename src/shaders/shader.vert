#version 450

layout (location = 0) out vec2 interp_uv;

layout (push_constant) uniform _matrices {
	mat4 mvp;
} matrices;

vec3 pos[3] = vec3[](
	vec3(0., -.5, 0.),
	vec3(-.5, .5, 0.),
	vec3(.5, .5, 0.)
);

vec2 uv[3] = vec2[](
	vec2(0.5, 1),
	vec2(0., 0.),
	vec2(1., 0.)
);

vec3 colours[3] = vec3[](
	vec3(1., 0., 0.),
	vec3(0., 1., 0.),
	vec3(0., 0., 1.)
);

void main() {
	gl_Position = matrices.mvp * vec4(pos[gl_VertexIndex], 1.);
	//interp_colour = colours[gl_VertexIndex];
	interp_uv = uv[gl_VertexIndex];
}
