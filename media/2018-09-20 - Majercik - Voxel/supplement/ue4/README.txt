Instructions for using our Ray-Box algorithm as a UE4 engine mod. Familiarity with our algorithm is assumed.

To run:
 - Copy ParticleGpuSimulation.cpp and FXSystemPrivate.h into the Engine/Source/Runtime/Engine/Private/Particles/ directory of your Unreal Engine 4 (UE4) clone.
 - Copy ParticleModuleRequired.h into Engine/Source/Runtime/Engine/Classes/Particles/.
 - Copy all the shader files (.usf,.ush) into Engine/Shaders/Private/.

Create a new UE4 project. Open the editor and create a GPU emitter. The GPU emitter will have a "Voxel" checkbox. Check that box, and the emitter should emit GPU particles (you may have to restart the editor for the change to take effect).

Alternate instructions:

You may wish to merge a lighter weight version of the mode base on your individual requirements. To make this easy, here is a rough breakdown of the modifications to each file with respect to our rendering algorithm. The files are listed in rough order of importance.

 - ParticleGPUSimulation.cpp: Most important file. Contains code for loading a voxel model from a binary, adds a Color texture to store voxel color, and controls the number of emitted voxels through the size bounds on the voxel state textures. NOTE: If you want more voxels, (or smaller state textures) you will have to change the GParticleSimulationTextureSizeX and GParticleSimulationTextureSizeY variables.
 - BasePassPixelShader.usf: With a preprocessor flag, sets world normal and depth based on our ray-box intersection (discarding no-hit pixels) when rendering particles from a GPU sprite emitter.
 - ParticleGPUSpriteVertexFactory.ush: Implements the voxel bounds calculation and positions the vertices of the quad accordingly. Reads changes vertex Color to be read off the voxel color texture.
 - BasePassVertexShader.usf: More logic related bounding box computation.
 - ParticleSimulationShader.usf, ParticleInjectionShader.usf: These are modified to render to and read from the Color texture, no other change to simulation/injection logic.
 - FXSystemPrivate.h: Adds the Color field to FNewParticle to read into the color texture on injection.
 - ParticleModuleRequired.h: Adds the "Voxel" checkbox and "Model Name" fields to the GPU sprite emitter.


