/* Ingo Wald's variation on Timo Aila's ray-box intersection to incorporate normalds and distances
   Personal communication July 2016  
*/
#ifndef ailaWald_glsl
#define ailaWald_glsl

#include "Box.glsl"
#include <g3dmath.glsl>

// Returns true if R intersects the oriented box. If there is an intersection, sets distance and normal based on the
// hit point. If no intersection, then distance and normal are undefined.
bool ailaWaldIntersectBox(Box box, Ray ray, out float distance, out vec3 normal, const in bool oriented, in vec3 _invRayDirection) {
    // Move to the box's reference frame
    ray.origin    = box.rotation * (ray.origin - box.center);
    if (oriented) {
        ray.direction = box.rotation * ray.direction;
    }
    // Need the inverse direction in the box's reference frame
    vec3 invRayDirection = oriented ? Vector3(safeInverse(ray.direction.x), safeInverse(ray.direction.y), safeInverse(ray.direction.z)) : _invRayDirection;

    vec3 t_min = (-box.radius - ray.origin) * invRayDirection;
    vec3 t_max = ( box.radius - ray.origin) * invRayDirection;
    float t0 = maxComponent(min(t_min, t_max));
    float t1 = minComponent(max(t_min, t_max));

    // Compute the intersection distance
    distance = (t0 > 0.0) ? t0 : t1;
    vec3 V = (ray.origin + ray.direction * distance) * box.invRadius;
    normal = vec3(0, 0, 0);
    int i = indexOfMaxComponent(abs(V));
    normal[i] = sign(V[i]) * (all(lessThan(abs(ray.origin), box.radius)) ? -1.0 : 1.0);
    if (oriented) {
        normal *= box.rotation;
    }

    return (t0 <= t1) && (distance > 0.0) && ! isinf(distance); 
}


// Returns true if R intersects the oriented box. If there is an intersection, sets distance and normal based on the
// hit point. If no intersection, then distance and normal are undefined.
bool ailaWaldHitAABox(vec3 boxCenter, vec3 boxRadius, vec3 rayOrigin, vec3 rayDirection, vec3 invRayDirection) {
    rayOrigin    -= boxCenter;

    vec3 t_min = (-boxRadius - rayOrigin) * invRayDirection;
    vec3 t_max = ( boxRadius - rayOrigin) * invRayDirection;
    float t0 = maxComponent(min(t_min, t_max));
    float t1 = minComponent(max(t_min, t_max));

    // Compute the intersection distance
    float distance = (t0 > 0.0) ? t0 : t1;
    
    return (t0 <= t1) && (distance > 0.0); 
}

#endif

