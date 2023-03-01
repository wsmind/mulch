#ifndef _VIEW_H_
#define _VIEW_H_

layout(binding = 0) uniform view_constants
{
    mat4 view_matrix;
    mat4 projection_matrix;
};

#endif // _VIEW_H_
