layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 1) buffer Lights {
    Light lights[];
};

const float PI = 3.141592653;
const float EPS = 1.0e-6;

const float PICTURE_ASPECT = 4.0 / 3.0;

struct Camera {
    vec3 position;
    vec3 direction;
    vec3 up;
    float fov;
    float aspect;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

Camera test_camera() {
    Camera camera;
    camera.position = vec3(-1.0, 2.5, 2.0);
    camera.direction = normalize(vec3(0.25) - camera.position);
    camera.up = vec3(0.0, 1.0, 0.0);
    camera.fov = PI / 4.0;
    camera.aspect = PICTURE_ASPECT;
    return camera;
}

Material test_material() {
    Material mat;
    mat.albedo = vec4(1.0);
    mat.roughness = 0.5;
    mat.reflectance = 0.25;
    mat.ambient_ratio = 0.02;
    return mat;
}

Ray camera_ray(in Camera camera, in vec2 uv) {
    Ray ray;
    ray.origin = camera.position;
    vec3 camera_dir = camera.direction;
    vec3 x_axis = normalize(cross(camera.direction, camera.up));
    vec3 y_axis = normalize(cross(x_axis, camera.direction));
    ray.direction = camera_dir / tan(camera.fov / 2.0);
    ray.direction += uv.x * camera.aspect * x_axis + uv.y * y_axis;
    ray.direction = normalize(ray.direction);
    return ray;
}

bool ray_marching(out vec3 position, out vec3 normal, in Ray ray) {
    float t = 1000.0;
    for (int i = 0; i < 3; i++) {
        float tmp = -ray.origin[i] / ray.direction[i];
        vec3 pos = ray.origin + tmp * ray.direction;
        bvec3 flag = bvec3(
            -EPS <= pos.x && pos.x <= 1.0 + EPS,
            -EPS <= pos.y && pos.y <= 1.0 + EPS,
            -EPS <= pos.z && pos.z <= 1.0 + EPS
        );
        if (0.0 < tmp && tmp < t && all(flag)) {
            t = tmp;
            position = pos;
            normal = vec3(0.0);
            normal[i] = -1.0;
        }
    }
    for (int i = 0; i < 3; i++) {
        float tmp = (1.0 - ray.origin[i]) / ray.direction[i];
        vec3 pos = ray.origin + tmp * ray.direction;
        bvec3 flag = bvec3(
            -EPS <= pos.x && pos.x <= 1.0 + EPS,
            -EPS <= pos.y && pos.y <= 1.0 + EPS,
            -EPS <= pos.z && pos.z <= 1.0 + EPS
        );
        if (0.0 < tmp && tmp < t && all(flag)) {
            t = tmp;
            position = pos;
            normal = vec3(0.0);
            normal[i] = 1.0;
        }
    }
    if (t > 900.0) return false;
    position = ray.origin + t * ray.direction;
    return true;
}

void main() {
    Camera camera = test_camera();
    Ray ray = camera_ray(camera, uv);

    vec3 position, normal;
    if(!ray_marching(position, normal, ray)) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    Light light = lights[0];
    Material mat = test_material();
    
    vec3 pre_color = microfacet_color(position, normal, light, ray.direction, mat);
    pre_color = clamp(pre_color, 0.0, 1.0);
    pre_color = ambient_correction(pre_color, mat);
    color = vec4(pre_color, 1.0);
}

