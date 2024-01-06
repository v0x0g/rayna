/* Kay-Kajiya method for ray-box intersection from
   Kay, T. L. and Kajiya, J. T., "Ray Tracing Complex Scenes", Computer Graphics, 20(4), pp. 269-78, August, 1986
*/
#ifndef kayKajiya_glsl
#define kayKajiya_glsl

#include "Box.glsl"
#include <g3dmath.glsl>

// Returns true if R intersects the oriented box. If there is an intersection, sets distance and normal based on the
// hit point. If no intersection, then distance and normal are undefined.
bool kayKajiyaIntersectBox(Box box, Ray ray, out float distance, out vec3 normal, const in bool oriented, in vec3 _invRayDirection) {
    // Move to the box's reference frame
    ray.origin    = box.rotation * (ray.origin - box.center);
    if (oriented) {
        ray.direction = box.rotation * ray.direction;
    }

    float tNear = -inf;
    float tFar = inf;
 
    // loop for each AABB plane (X,Y,Z)
    for (int i = 0; i < 3; ++i) {
        if (ray.direction[i] == 0.0) {
            // Parallel to plane 
            if ((ray.origin[i] < -box.radius[i]) || (ray.origin[i] > box.radius[i])) {
                return false;
            }
        } else {
            // intersection distances to plane.
            float recipDir = oriented ? (1.0 / ray.direction[i]) : _invRayDirection[i];
            float t1 = (-box.radius[i] - ray.origin[i]) * recipDir;
            float t2 = ( box.radius[i] - ray.origin[i]) * recipDir;

            if (t1 > t2)      swap(t1, t2);
            if (t1 > tNear)   tNear = t1;
            if (t2 < tFar)    tFar = t2;
            if (tNear > tFar) return false;
            if (tFar < 0.0)   return false;
        }
    }

    // Compute the intersection distance
    if (tNear >= 0.0) {
        distance = tNear;
    } else {
        distance = tFar;
    }
    vec3 V = (ray.origin + ray.direction * distance) * box.invRadius;
    normal = vec3(0, 0, 0);
    int i = indexOfMaxComponent(abs(V));
    normal[i] = sign(V[i]) * (all(lessThan(abs(ray.origin), box.radius)) ? -1.0 : 1.0);
    if (oriented) {
        normal *= box.rotation;
    }

    return true;
}


bool kayKajiyaHitAABox(vec3 boxCenter, vec3 boxRadius, vec3 rayOrigin, vec3 rayDirection, vec3 invRayDirection) {
    // Move to the box's reference frame
    rayOrigin    -= boxCenter;

    float tNear = -inf;
    float tFar = inf;
 
    // loop for each AABB plane (X,Y,Z)
    for (int i = 0; i < 3; ++i) {
        if (rayDirection[i] == 0.0) {
            // Parallel to plane 
            if ((rayOrigin[i] < -boxRadius[i]) || (rayOrigin[i] > boxRadius[i])) {
                return false;
            }
        } else {
            // intersection distances to plane.
            float recipDir = invRayDirection[i];
            float t1 = (-boxRadius[i] - rayOrigin[i]) * recipDir;
            float t2 = ( boxRadius[i] - rayOrigin[i]) * recipDir;

            if (t1 > t2)      swap(t1, t2);
            if (t1 > tNear)   tNear = t1;
            if (t2 < tFar)    tFar = t2;
            if (tNear > tFar) return false;
            if (tFar < 0.0)   return false;
        }
    }

    return true;
}
#endif

