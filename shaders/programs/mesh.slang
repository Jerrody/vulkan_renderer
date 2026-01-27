import modules;

static const let MAX_VERTICES : uint32_t = 64;
static const let MAX_TRIANGLES : uint32_t = 64;
static const let GROUP_SIZE : uint32_t = MAX_VERTICES;

groupshared Payload payload;

///////////////////////////////////////////////////// TASK //////////////////////////////////////////////////////////////

[shader("amplification")]
[numthreads(1, 1, 1)]
func main(const uint32_t group_id: SV_GroupID)
{
    const let instance_object = push_constants.ptr_instance_object[group_id];

    payload = Payload(group_id);

    DispatchMesh(instance_object.meshlet_count, 1, 1, payload);
}

///////////////////////////////////////////////////// MESH //////////////////////////////////////////////////////////////

[shader("mesh")]
[outputtopology("triangle")]
[numthreads(GROUP_SIZE, 1, 1)]
func main(
    const uint group_id: SV_GroupID,
    const uint group_index: SV_GroupIndex,
    OutputVertices<VertexOutput, MAX_VERTICES> outVerts,
    OutputIndices<uint3, MAX_TRIANGLES> outIndices,
    OutputPrimitives<PrimitiveData, MAX_TRIANGLES> out_primitives)
{
    const let instance_object = push_constants.ptr_instance_object[payload.instance_object_index];
    let ptr_mesh_object = instance_object.ptr_mesh_object;
    let meshlet = ptr_mesh_object.meshlets[group_id];

    SetMeshOutputCounts(meshlet.vertex_count, meshlet.triangle_count);

    if (group_index < meshlet.vertex_count)
    {
        let vertexLookupIndex = meshlet.vertex_offset + group_index;
        let globalVertexId = ptr_mesh_object.vertex_indices[vertexLookupIndex];

        let v = ptr_mesh_object.vertices[globalVertexId];
        outVerts[group_index] = process_vertex(v,
                                               instance_object.model_matrix,
                                               push_constants.view_projection_matrix);
    }

    if (group_index < meshlet.triangle_count)
    {
        let triangleLookupIndex = meshlet.triangle_offset + (group_index * 3);
        let i0 = ptr_mesh_object.local_indices[triangleLookupIndex + 0];
        let i1 = ptr_mesh_object.local_indices[triangleLookupIndex + 1];
        let i2 = ptr_mesh_object.local_indices[triangleLookupIndex + 2];

        outIndices[group_index] = uint32_t3(i0, i1, i2);

        out_primitives[group_index] = PrimitiveData(instance_object.device_address_material);
    }
}

[ForceInline]
func process_vertex(const v: Vertex, const model_matrix: float4x4, const view_projection_matrix: float4x4)->VertexOutput
{
    let position = float4(v.position, 1.0);
    let world_position = mul(model_matrix, position);
    let clip_position = mul(view_projection_matrix, world_position);

    let vertex_output : VertexOutput = VertexOutput(clip_position, v.normal, v.uv);

    return vertex_output;
}

///////////////////////////////////////////////////// FRAGMENT //////////////////////////////////////////////////////////////

[shader("fragment")]
func main(const vertex_output: VertexOutput, const primitive_data: PrimitiveData)->float4
{
    let material = primitive_data.device_address_material;
    const let surface_data = material.eval(SurfaceData(), vertex_output.uv);

    const let final_color = surface_data.color;

    return final_color;
}
