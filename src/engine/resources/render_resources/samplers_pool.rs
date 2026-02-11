use vulkanite::vk::{rs::*, *};

#[derive(Default, Clone, Copy)]
pub struct SamplerReference {
    pub index: usize,
    pub generation: usize,
}

impl SamplerReference {
    pub fn get_sampler(&self, samplers_pool: &SamplersPool) -> Option<Sampler> {
        samplers_pool.get_sampler(*self)
    }
}

#[derive(Default)]
struct SamplerSlot {
    pub sampler: Option<Sampler>,
    pub generation: usize,
}

pub struct SamplersPool {
    device: Device,
    slots: Vec<SamplerSlot>,
    free_indices: Vec<usize>,
}

impl SamplersPool {
    pub fn new(device: Device) -> Self {
        let slots = (0..16).into_iter().map(|_| Default::default()).collect();

        Self {
            device,
            slots,
            free_indices: (0..16).rev().collect(),
        }
    }

    pub fn create_sampler(
        &mut self,
        filter: Filter,
        wrap: SamplerAddressMode,
        mip_map_enabled: bool,
    ) -> SamplerReference {
        let mipmap_mode = if mip_map_enabled {
            match filter {
                Filter::Nearest => SamplerMipmapMode::Nearest,
                Filter::Linear => SamplerMipmapMode::Linear,
                _ => panic!("Unsupported filter mode: {:?}", filter),
            }
        } else {
            SamplerMipmapMode::Nearest
        };

        let compare_op = if mip_map_enabled {
            CompareOp::Always
        } else {
            CompareOp::Never
        };

        let max_lod = if mip_map_enabled {
            LOD_CLAMP_NONE
        } else {
            Default::default()
        };

        let sampler_create_info = SamplerCreateInfo {
            mag_filter: filter,
            min_filter: filter,
            mipmap_mode,
            address_mode_u: wrap,
            address_mode_v: wrap,
            address_mode_w: wrap,
            compare_op: compare_op,
            max_lod: max_lod,
            ..Default::default()
        };
        let sampler = self.device.create_sampler(&sampler_create_info).unwrap();

        self.insert_sampler(sampler)
    }

    fn insert_sampler(&mut self, sampler: Sampler) -> SamplerReference {
        let index = self.free_indices.pop().unwrap();

        let sampler_slot = unsafe { self.slots.get_mut(index).unwrap_unchecked() };
        sampler_slot.sampler = Some(sampler);
        sampler_slot.generation += 1;

        let generation = sampler_slot.generation;

        SamplerReference { index, generation }
    }

    fn get_sampler(&self, sampler_reference: SamplerReference) -> Option<Sampler> {
        let mut sampler = None;

        let slot = unsafe { self.slots.get(sampler_reference.index).unwrap_unchecked() };
        if slot.generation == sampler_reference.generation {
            sampler = slot.sampler.to_owned();
        }

        sampler
    }

    pub fn destroy_samplers(&mut self) {
        self.slots.drain(..).for_each(|sampler_slot| unsafe {
            self.device.destroy_sampler(sampler_slot.sampler);
        });
    }
}
