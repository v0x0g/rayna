/* 
Fast Ray-Box Intersection
by Andrew Woo
from "Graphics Gems", Academic Press, 1990

Extended with a precomputed inverse ray direction, oriented box tests, and Wald's method for
finding the normal at the intersection point.
*/
#ifndef woo_glsl
#define woo_glsl

#include "Box.glsl"
#include <g3dmath.glsl>

#define NUMDIM	3
#define RIGHT	0
#define LEFT	1
#define MIDDLE	2


bool wooHelper(Box box, Ray ray, const bool rayIsOutsideBox, in vec3 _invRayDirection, const in bool oriented, out float distance, out vec3 normal) {
    vec3 coord;

    // Move to the box's reference frame
    ray.origin = box.rotation * (ray.origin - box.center);
    if (oriented) {
        ray.direction = box.rotation * ray.direction;
    }

    // Need the inverse direction in the box's reference frame
    vec3 invRayDirection = oriented ? Vector3(safeInverse(ray.direction.x), safeInverse(ray.direction.y), safeInverse(ray.direction.z)) : _invRayDirection;

    vec3 minB = -box.radius;
    vec3 maxB = +box.radius;

    vec3 quadrant;

    int whichPlane;
    vec3 maxT;
    vec3 candidatePlane;

    if (! rayIsOutsideBox) {
        bool inside = true;
        
        // Find candidate planes; this loop can be avoided if
        //    rays cast all from the eye(assume perpsective view)
        for (int i = 0; i < 3; ++i) {
            if (ray.origin[i] < minB[i]) {
                quadrant[i] = LEFT;
                candidatePlane[i] = minB[i];
                inside = false;
            } else if (ray.origin[i] > maxB[i]) {
                quadrant[i] = RIGHT;
                candidatePlane[i] = maxB[i];
                inside = false;
            } else {
                quadrant[i] = MIDDLE;
            }
        }
    
	    /* Ray origin inside bounding box */
	    if (inside) {
            // Result is wrong in this case because Woo doesn't actually find the hit point
            coord = ray.origin;
            distance = 0;
            normal = -ray.direction;
            return true;
	    }
    }


    /* Calculate T distances to candidate planes */
    for (int i = 0; i < 3; ++i) {
        if ((quadrant[i] != MIDDLE) && (ray.direction[i] != 0.0)) {
            maxT[i] = (candidatePlane[i] - ray.origin[i]) * invRayDirection[i];
        } else {
            maxT[i] = -1.0;
        }
    }

    /* Get largest of the maxT's for final choice of intersection */
    whichPlane = 0;
    for (int i = 1; i < 3; i++) {
        if (maxT[whichPlane] < maxT[i]) {
            whichPlane = i;
        }
    }

    /* Check final candidate actually inside box */
    if (maxT[whichPlane] < 0.0) return false;
    
    for (int i = 0; i < 3; ++i) {
        if (whichPlane != i) {
            coord[i] = ray.origin[i] + maxT[whichPlane] * ray.direction[i];
            if (coord[i] < minB[i] || coord[i] > maxB[i]) {
                return false;
            }
        } else {
            coord[i] = candidatePlane[i];
        }
    }

    distance = maxT[whichPlane];
    
    // Use Wald's method for recovering the normal (note that coord is in the box's reference frame already)
    normal = vec3(0, 0, 0);
    int i = indexOfMaxComponent(abs(coord));
    normal[i] = sign(coord[i]) * (all(lessThan(abs(ray.origin), box.radius)) ? -1.0 : 1.0);
    if (oriented) {
        normal *= box.rotation;
    }

    /* ray hits box */
    return true;
}	


bool wooIntersectBox(Box box, Ray ray, out float distance, out vec3 normal, const in bool oriented, in vec3 _invRayDirection) {
    return wooHelper(box, ray, false, _invRayDirection, oriented, distance, normal);
}


bool wooHitAABox(vec3 boxCenter, vec3 boxRadius, vec3 rayOrigin, vec3 rayDirection, vec3 invRayDirection) {
    float ignoreDistance;
    vec3 ignoreNormal;
    return wooHelper(Box(boxCenter, boxRadius, vec3(1.0) / boxRadius, mat3(1.0)), Ray(rayOrigin, rayDirection), false, invRayDirection, false, ignoreDistance, ignoreNormal);
}

#endif

