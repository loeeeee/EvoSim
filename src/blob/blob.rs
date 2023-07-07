use std::f32::NAN;

use bevy::prelude::*;

/// flag of a blob entity
#[derive(Component)]
pub struct Blob;

#[derive(Component, Clone)]
pub struct BlobInfo{
    pub center: Vec2,
    // bound: [min,max] to represent size
    pub xbound: [f32;2],
    pub ybound: [f32;2],
    pub color: Color
}

impl Default for BlobInfo {
    fn default() -> Self {
        Self {
            center: Vec2::NAN,
            xbound:[NAN,NAN],
            ybound:[NAN,NAN],
            color: Color::ALICE_BLUE
        }
    }
}

impl BlobInfo {
    pub fn init(&mut self, center:Vec2, size:Vec2){
        self.center = center;
        self.xbound = [center.x-size.x,center.x+size.x];
        self.ybound = [center.y-size.y,center.y+size.y];
    }

    /// Add geometric infomation of new blocks in blob,
    /// update blobinfo accordingly
    pub fn add(&mut self, translation:Vec2, size:Vec2){
        let large = translation + size;
        let small = translation - size;

        self.xbound[0] = self.xbound[0].min(small.x);
        self.xbound[1] = self.xbound[1].max(large.x);
        self.ybound[0] = self.ybound[0].min(small.y);
        self.ybound[1] = self.ybound[1].max(large.y);
    }
}

#[derive(Bundle)]
pub struct BlobBundle{
    // flag
    blob_flag:Blob,

    // set visibility so it's children can be seen
    visibility: Visibility,
    computed_visibility: ComputedVisibility,

    // identity transform
    transform_bundle:TransformBundle,

    // real blob information
    info: BlobInfo
}

impl Default for BlobBundle {
    fn default() -> Self {
        Self {
            blob_flag: Blob,
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::HIDDEN,
            transform_bundle: TransformBundle::IDENTITY,
            info: BlobInfo::default()
        }
    }
}