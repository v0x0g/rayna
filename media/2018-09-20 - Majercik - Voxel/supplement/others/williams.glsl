/* Amy Williams jgt ray-box intersection   
  http://people.csail.mit.edu/amy/papers/box-jgt.pdf

  Amy Williams, Steve Barrus, R. Keith Morley, Peter Shirley,
  An Efficient and Robust Ray–Box Intersection Algorithm,
  jgt, 2004

*/
#ifndef williams_glsl
#define williams_glsl

#include "Box.glsl"
#include <g3dmath.glsl>


// Returns true if R intersects the oriented box. If there is an intersection, sets distance and normal based on the
// hit point. If no intersection, then distance and normal are undefined.
bool williamsIntersectBox(Box box, Ray ray, out float distance, out vec3 normal, const in bool oriented, in vec3 _invRayDirection) {
    // Move to the box's reference frame
    ray.origin    = box.rotation * (ray.origin - box.center);

    if (oriented) {
        ray.direction = box.rotation * ray.direction;
    }

    // Need the inverse direction in the box's reference frame
    vec3 invRayDirection = oriented ? (1.0 / ray.direction) : _invRayDirection;

    float tmin, tmax, tymin, tymax, tzmin, tzmax;

    if (invRayDirection.x >= 0.0) {
        tmin = (-box.radius.x - ray.origin.x) * invRayDirection.x;
        tmax = ( box.radius.x - ray.origin.x) * invRayDirection.x;
    } else {
        tmin = ( box.radius.x - ray.origin.x) * invRayDirection.x;
        tmax = (-box.radius.x - ray.origin.x) * invRayDirection.x;
    }

    if (invRayDirection.y >= 0.0) {
        tymin = (-box.radius.y - ray.origin.y) * invRayDirection.y;
        tymax = ( box.radius.y - ray.origin.y) * invRayDirection.y;
    } else {                                   
        tymin = ( box.radius.y - ray.origin.y) * invRayDirection.y;
        tymax = (-box.radius.y - ray.origin.y) * invRayDirection.y;
    }

    if ((tmin > tymax) || (tymin > tmax)) {
        return false;
    }

    if (tymin > tmin) {
        tmin = tymin;
    }

    if (tymax < tmax) {
        tmax = tymax;
    }

    if (invRayDirection.z >= 0.0) {
        tzmin = (-box.radius.z - ray.origin.z) * invRayDirection.z;
        tzmax = ( box.radius.z - ray.origin.z) * invRayDirection.z;
    } else {
        tzmin = ( box.radius.z - ray.origin.z) * invRayDirection.z;
        tzmax = (-box.radius.z - ray.origin.z) * invRayDirection.z;
    }

    if ((tmin > tzmax) || (tzmin > tmax)) {
        return false;
    }

    if (tzmin > tmin) {
        tmin = tzmin;
    }

    if (tzmax < tmax) {
        tmax = tzmax;
    }

    // Find the normal and distance using Wald's method
    distance = (tmin > 0) ? tmin : tmax; 
    vec3 V = (ray.origin + ray.direction * distance) * box.invRadius;
    normal = vec3(0, 0, 0);
    int i = indexOfMaxComponent(abs(V));
    normal[i] = sign(V[i]) * (all(lessThan(abs(ray.origin), box.radius)) ? -1.0 : 1.0);

    if (oriented) {
        normal *= box.rotation;
    }

    return (distance > 0.0) && ! isinf(distance); 
}


bool williamsHitAABox(vec3 boxCenter, vec3 boxRadius, vec3 rayOrigin, vec3 rayDirection, vec3 invRayDirection) {
    // Move to the box's reference frame
    rayOrigin -= boxCenter;

    float tmin, tmax, tymin, tymax, tzmin, tzmax;

    if (invRayDirection.x >= 0.0) {
        tmin = (-boxRadius.x - rayOrigin.x) * invRayDirection.x;
        tmax = ( boxRadius.x - rayOrigin.x) * invRayDirection.x;
    } else {
        tmin = ( boxRadius.x - rayOrigin.x) * invRayDirection.x;
        tmax = (-boxRadius.x - rayOrigin.x) * invRayDirection.x;
    }

    if (invRayDirection.y >= 0.0) {
        tymin = (-boxRadius.y - rayOrigin.y) * invRayDirection.y;
        tymax = ( boxRadius.y - rayOrigin.y) * invRayDirection.y;
    } else {                                   
        tymin = ( boxRadius.y - rayOrigin.y) * invRayDirection.y;
        tymax = (-boxRadius.y - rayOrigin.y) * invRayDirection.y;
    }

    if ((tmin > tymax) || (tymin > tmax)) {
        return false;
    }

    if (tymin > tmin) {
        tmin = tymin;
    }

    if (tymax < tmax) {
        tmax = tymax;
    }

    if (invRayDirection.z >= 0.0) {
        tzmin = (-boxRadius.z - rayOrigin.z) * invRayDirection.z;
        tzmax = ( boxRadius.z - rayOrigin.z) * invRayDirection.z;
    } else {
        tzmin = ( boxRadius.z - rayOrigin.z) * invRayDirection.z;
        tzmax = (-boxRadius.z - rayOrigin.z) * invRayDirection.z;
    }

    if ((tmin > tzmax) || (tzmin > tmax)) {
        return false;
    }

    if (tzmin > tmin) {
        tmin = tzmin;
    }

    if (tzmax < tmax) {
        tmax = tzmax;
    }

    float distance = (tmin > 0) ? tmin : tmax;

    return distance > 0.0; 
}
#endif

