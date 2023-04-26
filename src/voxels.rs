use glam::*;

pub type Coords = (usize, usize, usize);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexData {
    pub position: Vec3,
    pub normal: Vec3,
}

pub struct VoxelGrid {
    data: [u64; 64 * 64],
}

impl VoxelGrid {
    pub fn new() -> Self {
        let data = [0u64; 64 * 64];

        Self { data }
    }

    pub fn read(&self, (x, y, z): Coords) -> u64 {
        let line = self.data[z * 64 + y];
        (line & (1 << x)) >> x
    }

    pub fn add(&mut self, other: &Self) {
        for z in 0..64 {
            for y in 0..64 {
                let index = z * 64 + y;
                self.data[index] |= other.data[index];
            }
        }
    }

    pub fn subtract(&mut self, other: &Self) {
        for z in 0..64 {
            for y in 0..64 {
                let index = z * 64 + y;
                self.data[index] &= !other.data[index];
            }
        }
    }

    pub fn paint_cube(&mut self, min: Coords, max: Coords) {
        let min_mask: u64 = (1 << min.0) - 1;
        let max_mask: u64 = (1 << max.0) - 1;
        let mask: u64 = max_mask - min_mask;
        for z in min.2..=max.2 {
            for y in min.1..=max.1 {
                self.data[z * 64 + y] |= mask;
            }
        }
    }

    pub fn paint_sphere(&mut self, pos: Coords, radius: f32) {
        let bounds_radius = radius.ceil() as i32;

        for z in -bounds_radius..=bounds_radius {
            for y in -bounds_radius..=bounds_radius {
                for x in -bounds_radius..=bounds_radius {
                    let point = (x as f32, y as f32, z as f32);
                    let distance =
                        (point.0 * point.0 + point.1 * point.1 + point.2 * point.2).sqrt();
                    if distance <= radius {
                        let voxel_pos = (
                            (pos.0 as i32 + x) as usize,
                            (pos.1 as i32 + y) as usize,
                            (pos.2 as i32 + z) as usize,
                        );
                        self.data[voxel_pos.2 * 64 + voxel_pos.1] |= 1 << voxel_pos.0;
                    }
                }
            }
        }
    }

    pub fn generate_mesh(&self) -> (Vec<VertexData>, Vec<u32>) {
        let mut vertices: Vec<VertexData> = vec![];
        let mut indices: Vec<u32> = vec![];

        let mut index_grid = vec![0u32; 64 * 64 * 64];

        for z in 0..63 {
            for y in 0..63 {
                for x in 0..63 {
                    let mut count = 0;
                    count += self.read((x, y, z));
                    count += self.read((x + 1, y, z));
                    count += self.read((x + 1, y + 1, z));
                    count += self.read((x, y + 1, z));
                    count += self.read((x, y, z + 1));
                    count += self.read((x + 1, y, z + 1));
                    count += self.read((x + 1, y + 1, z + 1));
                    count += self.read((x, y + 1, z + 1));

                    if count > 0 && count < 8 {
                        // generate a vertex here
                        let index = vertices.len();
                        index_grid[z * 64 * 64 + y * 64 + x] = index as u32;
                        vertices.push(VertexData {
                            position: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5),
                            normal: Vec3::ZERO,
                        });
                    }
                }
            }
        }

        // scan all interior edges
        for z in 1..63 {
            for y in 1..63 {
                for x in 0..63 {
                    let v0 = self.read((x, y, z));
                    let v1 = self.read((x + 1, y, z));

                    if v0 != v1 {
                        let i0 = index_grid[(z - 1) * 64 * 64 + (y - 1) * 64 + x];
                        let i1 = index_grid[z * 64 * 64 + (y - 1) * 64 + x];
                        let i2 = index_grid[z * 64 * 64 + y * 64 + x];
                        let i3 = index_grid[(z - 1) * 64 * 64 + y * 64 + x];

                        let mut normal = Vec3::new(1.0, 0.0, 0.0);
                        if v0 < v1 {
                            normal.x = -normal.x;
                            indices.extend([i0, i1, i2, i2, i3, i0]);
                        } else {
                            indices.extend([i0, i3, i2, i2, i1, i0]);
                        }

                        vertices[i0 as usize].normal += normal;
                        vertices[i1 as usize].normal += normal;
                        vertices[i2 as usize].normal += normal;
                        vertices[i3 as usize].normal += normal;
                    }
                }
            }
        }

        for z in 1..63 {
            for x in 1..63 {
                for y in 0..63 {
                    let v0 = self.read((x, y, z));
                    let v1 = self.read((x, y + 1, z));

                    if v0 != v1 {
                        let i0 = index_grid[(z - 1) * 64 * 64 + y * 64 + (x - 1)];
                        let i1 = index_grid[(z - 1) * 64 * 64 + y * 64 + x];
                        let i2 = index_grid[z * 64 * 64 + y * 64 + x];
                        let i3 = index_grid[z * 64 * 64 + y * 64 + (x - 1)];

                        let mut normal = Vec3::new(0.0, 1.0, 0.0);
                        if v0 < v1 {
                            normal.y = -normal.y;
                            indices.extend([i0, i1, i2, i2, i3, i0]);
                        } else {
                            indices.extend([i0, i3, i2, i2, i1, i0]);
                        }

                        vertices[i0 as usize].normal += normal;
                        vertices[i1 as usize].normal += normal;
                        vertices[i2 as usize].normal += normal;
                        vertices[i3 as usize].normal += normal;
                    }
                }
            }
        }

        for y in 1..63 {
            for x in 1..63 {
                for z in 0..63 {
                    let v0 = self.read((x, y, z));
                    let v1 = self.read((x, y, z + 1));

                    if v0 != v1 {
                        let i0 = index_grid[z * 64 * 64 + (y - 1) * 64 + (x - 1)];
                        let i1 = index_grid[z * 64 * 64 + y * 64 + (x - 1)];
                        let i2 = index_grid[z * 64 * 64 + y * 64 + x];
                        let i3 = index_grid[z * 64 * 64 + (y - 1) * 64 + x];

                        let mut normal = Vec3::new(0.0, 0.0, 1.0);
                        if v0 < v1 {
                            normal.z = -normal.z;
                            indices.extend([i0, i1, i2, i2, i3, i0]);
                        } else {
                            indices.extend([i0, i3, i2, i2, i1, i0]);
                        }

                        vertices[i0 as usize].normal += normal;
                        vertices[i1 as usize].normal += normal;
                        vertices[i2 as usize].normal += normal;
                        vertices[i3 as usize].normal += normal;
                    }
                }
            }
        }

        (vertices, indices)
    }
}
