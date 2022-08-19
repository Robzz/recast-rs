#[derive(Debug)]
pub struct Mesh<'a> {
    pub(crate) vertices: &'a [f32],
    pub(crate) indices: Option<&'a [i32]>
}

impl<'a> Mesh<'a> {
    // TODO: checks to make sure that the geometry is not obviously wrong (e.g. not multiple of 3
    // for triangles)
    pub fn from_vertex_buffer(vertices: &'a [f32]) -> Result<Mesh<'a>, String> {
        if vertices.len() > std::i32::MAX as usize {
            return Err(format!("The maximum support side for vertices and indices buffers is {}, split the mesh in smaller pieces.", std::i32::MAX));
        }

        Ok(Mesh { vertices, indices: None })
    }

    pub fn from_buffers(vertices: &'a [f32], indices: &'a [i32]) -> Result<Mesh<'a>, String> {
        if vertices.len() > std::i32::MAX as usize || indices.len() > std::i32::MAX as usize {
            return Err(format!("The maximum support side for vertices and indices buffers is {}, split the mesh in smaller pieces.", std::i32::MAX));
        }

        Ok(Mesh { vertices, indices: Some(indices) })
    }
}

#[derive(Debug)]
pub struct MarkedMesh<'a> {
    pub(crate) vertices: &'a [f32],
    pub(crate) indices: Vec<i32>,
    pub(crate) areas: Vec<u8>
}
