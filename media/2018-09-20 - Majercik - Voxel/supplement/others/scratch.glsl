/* scratchapixel's version of Williams et al. from 
   http://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection
   viewed July 14, 2015
*/
#ifndef scratch_glsl
#define scratch_glsl

#include "Box.glsl"
#include <g3dmath.glsl>


// Returns true if R intersects the oriented box. If there is an intersection, sets distance and normal based on the
// hit point. If no intersection, then distance and normal are undefined.
bool scratchIntersectBox(Box box, Ray ray, out float distance, out vec3 normal, const in bool oriented, in vec3 _invRayDirection) {
    // Move to the box's reference frame
    ray.origin    = box.rotation * (ray.origin - box.center);

    if (oriented) {
        ray.direction = box.rotation * ray.direction;
    }

    // Need the inverse direction in the box's reference frame
    vec3 invRayDirection = oriented ? (1.0 / ray.direction) : _invRayDirection;

    float tmin, tmax, tymin, tymax, tzmin, tzmax;
    vec3 bounds[2];
    bounds[0] = -box.radius;
    bounds[1] = box.radius;

    ivec3 sgn;
    sgn[0] = (invRayDirection.x < 0) ? 1 : 0; 
    sgn[1] = (invRayDirection.y < 0) ? 1 : 0; 
    sgn[2] = (invRayDirection.z < 0) ? 1 : 0; 

    tmin  = (bounds[sgn[0]].x - ray.origin.x) * invRayDirection.x; 
    tmax  = (bounds[1 - sgn[0]].x - ray.origin.x) * invRayDirection.x; 
    tymin = (bounds[sgn[1]].y - ray.origin.y) * invRayDirection.y; 
    tymax = (bounds[1 - sgn[1]].y - ray.origin.y) * invRayDirection.y; 
 
    if ((tmin > tymax) || (tymin > tmax))  return false; 
    if (tymin > tmin) tmin = tymin; 
    if (tymax < tmax) tmax = tymax; 
 
    tzmin = (bounds[sgn[2]].z - ray.origin.z) * invRayDirection.z; 
    tzmax = (bounds[1 - sgn[2]].z - ray.origin.z) * invRayDirection.z; 
 
    if ((tmin > tzmax) || (tzmin > tmax)) return false; 
    if (tzmin > tmin) tmin = tzmin; 
    if (tzmax < tmax) tmax = tzmax; 
 
    // Find the normal and distance using Wald's method
    distance = (tmin > 0.0) ? tmin : tmax;
    vec3 V = (ray.origin + ray.direction * distance) * box.invRadius;
    normal = vec3(0, 0, 0);
    int i = indexOfMaxComponent(abs(V));
    normal[i] = sign(V[i]) * (all(lessThan(abs(ray.origin), box.radius)) ? -1.0 : 1.0);
    if (oriented) {
        normal *= box.rotation;
    }

    return (distance > 0.0) && ! isinf(distance); 
} 


bool scratchHitAABox(vec3 boxCenter, vec3 boxRadius, vec3 rayOrigin, vec3 rayDirection, vec3 invRayDirection) {
    rayOrigin -= boxCenter;

    float tmin, tmax, tymin, tymax, tzmin, tzmax;
    vec3 bounds[2];
    bounds[0] = -boxRadius;
    bounds[1] = boxRadius;

    ivec3 sgn;
    sgn[0] = (invRayDirection.x < 0) ? 1 : 0; 
    sgn[1] = (invRayDirection.y < 0) ? 1 : 0; 
    sgn[2] = (invRayDirection.z < 0) ? 1 : 0; 

    tmin  = (bounds[sgn[0]].x - rayOrigin.x) * invRayDirection.x; 
    tmax  = (bounds[1 - sgn[0]].x - rayOrigin.x) * invRayDirection.x; 
    tymin = (bounds[sgn[1]].y - rayOrigin.y) * invRayDirection.y; 
    tymax = (bounds[1 - sgn[1]].y - rayOrigin.y) * invRayDirection.y; 
 
    if ((tmin > tymax) || (tymin > tmax))  return false; 
    if (tymin > tmin) tmin = tymin; 
    if (tymax < tmax) tmax = tymax; 
 
    tzmin = (bounds[sgn[2]].z - rayOrigin.z) * invRayDirection.z; 
    tzmax = (bounds[1 - sgn[2]].z - rayOrigin.z) * invRayDirection.z; 
 
    if ((tmin > tzmax) || (tzmin > tmax)) return false; 
    if (tzmin > tmin) tmin = tzmin; 
    if (tzmax < tmax) tmax = tzmax; 
 
    float distance = (tmin > 0.0) ? tmin : tmax;
    return distance > 0.0; 
}
#endif

