use bevy_ecs::{
    resource::Resource,
    system::{Res, ResMut, SystemParam},
};
use vulkanite::vk::{
    CompareOp, Filter, LOD_CLAMP_NONE, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode,
    rs::{Device, Sampler},
};

#[derive(SystemParam)]
pub struct Samplers<'w> {
    samplers_pool: Res<'w, SamplersPool>,
}

impl<'w> Samplers<'w> {
    #[inline(always)]
    pub fn get(&self, sampler_reference: SamplerReference) -> Option<Sampler> {
        self.samplers_pool.get_sampler(sampler_reference)
    }
}

#[derive(SystemParam)]
pub struct SamplersMut<'w> {
    samplers_pool: ResMut<'w, SamplersPool>,
}

impl<'w> SamplersMut<'w> {
    #[inline(always)]
    pub fn get(&self, sampler_reference: SamplerReference) -> Option<Sampler> {
        self.samplers_pool.get_sampler(sampler_reference)
    }

    #[inline(always)]
    pub fn create_sampler(
        &mut self,
        filter: Filter,
        wrap: SamplerAddressMode,
        mip_map_enabled: bool,
    ) -> SamplerReference {
        self.samplers_pool
            .create_sampler(filter, wrap, mip_map_enabled)
    }
}

#[derive(Default, Clone, Copy)]
pub struct SamplerReference {
    pub index: u32,
    pub generation: u32,
}

#[derive(Default)]
struct SamplerSlot {
    pub sampler: Option<Sampler>,
    pub generation: u32,
}

#[derive(Resource)]
pub struct SamplersPool {
    device: Device,
    slots: Vec<SamplerSlot>,
    free_indices: Vec<u32>,
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

        let sampler_slot = unsafe { self.slots.get_mut(index as usize).unwrap_unchecked() };
        sampler_slot.sampler = Some(sampler);
        sampler_slot.generation += 1;

        let generation = sampler_slot.generation;

        SamplerReference { index, generation }
    }

    fn get_sampler(&self, sampler_reference: SamplerReference) -> Option<Sampler> {
        let mut sampler = None;

        let slot = unsafe {
            self.slots
                .get(sampler_reference.index as usize)
                .unwrap_unchecked()
        };
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
