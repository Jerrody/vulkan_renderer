use bevy_ecs::{
    resource::Resource,
    system::{Res, ResMut, SystemParam},
};
use slotmap::SlotMap;
use vulkanite::vk::{
    CompareOp, Filter, LOD_CLAMP_NONE, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode,
    rs::{Device, Sampler},
};

use crate::engine::ecs::SamplerKey;

#[derive(SystemParam)]
pub struct Samplers<'w> {
    samplers_pool: Res<'w, SamplersPool>,
}

impl<'w> Samplers<'w> {
    #[inline(always)]
    pub fn get(&self, sampler_reference: SamplerReference) -> Option<&Sampler> {
        self.samplers_pool.get_sampler(sampler_reference)
    }
}

#[derive(SystemParam)]
pub struct SamplersMut<'w> {
    samplers_pool: ResMut<'w, SamplersPool>,
}

impl<'w> SamplersMut<'w> {
    #[inline(always)]
    pub fn get(&self, sampler_reference: SamplerReference) -> Option<&Sampler> {
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
    pub key: SamplerKey,
}

impl SamplerReference {
    pub fn get_index(&self) -> u32 {
        self.key.0.get_key() - 1
    }
}

#[derive(Resource)]
pub struct SamplersPool {
    device: Device,
    slots: SlotMap<SamplerKey, Sampler>,
}

impl SamplersPool {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            slots: SlotMap::with_capacity_and_key(16),
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
            compare_op,
            max_lod,
            ..Default::default()
        };
        let sampler = self.device.create_sampler(&sampler_create_info).unwrap();

        self.insert_sampler(sampler)
    }

    fn insert_sampler(&mut self, sampler: Sampler) -> SamplerReference {
        let sampler_key = self.slots.insert(sampler);

        SamplerReference { key: sampler_key }
    }

    fn get_sampler(&self, sampler_reference: SamplerReference) -> Option<&Sampler> {
        self.slots.get(sampler_reference.key)
    }

    pub fn destroy_samplers(&mut self) {
        self.slots.drain().for_each(|(_, sampler)| unsafe {
            self.device.destroy_sampler(Some(sampler));
        });
    }
}
