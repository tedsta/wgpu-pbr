#!/usr/bin/env sh
glslangValidator shaders/tex_unlit.vert -V -l -o src/mesh/shaders/tex_unlit_vert.spv
glslangValidator shaders/tex_unlit.frag -V -l -o src/mesh/shaders/tex_unlit_frag.spv

glslangValidator shaders/pbr.vert -V -l -o src/mesh/shaders/pbr_vert.spv
glslangValidator shaders/untex_pbr.frag -V -l -o src/mesh/shaders/untex_pbr_frag.spv
glslangValidator shaders/tex_pbr.frag -V -l -o src/mesh/shaders/tex_pbr_frag.spv
glslangValidator shaders/tex_norm.frag -V -l -o src/mesh/shaders/tex_norm_frag.spv
glslangValidator shaders/tex_norm_pbr.frag -V -l -o src/mesh/shaders/tex_norm_pbr_frag.spv
glslangValidator shaders/tex_emiss_pbr.frag -V -l -o src/mesh/shaders/tex_emiss_pbr_frag.spv
