use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::consts::*;

use super::block::{PhysiBlockBundle, BlockAnchors};

/// flag of a blob entity
#[derive(Component)]
pub struct Blob;

#[derive(Debug)]
pub struct BlobBlock {
    id: Entity,
    top: Option<usize>,
    bottom: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
    vec_index: usize,
    size: Vec2,
    translation: Vec2,
    anchors: BlockAnchors
}

pub struct BlobBuilder<'a>{
    blob: Entity,
    color: Color,
    commands: Commands<'a, 'a>,
    pub blocks: Vec<BlobBlock>,
    current_pos: Option<usize>
}

impl<'a> BlobBuilder<'a> {
    /// BlobBuilder taks ownership of Commands, 
    /// which means you can not use Commands anymore after using the BlobBuilder.
    /// To use commands, you need to preform it before creating BlobBuilder
    /// or just create another system.
    /// 
    /// To generate multiple blobs, or want to use BlobBuilder in loops,
    /// please use [`clean()`] so that there won't be joints connects.
    pub fn from_commands(mut commands: Commands<'a, 'a>) -> Self{
        Self{
            blob: commands.spawn((
                Blob,
                Visibility::Visible,
                ComputedVisibility::HIDDEN,
                TransformBundle::IDENTITY
            )).id(),
            color: Color::AZURE,
            commands: commands,
            blocks: Vec::new(),
            current_pos:None
        }
    }

    /// set color for blob
    pub fn set_color(&mut self, color: Color) -> &mut Self{
        self.color = color;
        self
    }

    /// Clean all the things inside BlobBuilder
    /// Equvalent to drop the old builder and generate a new one
    pub fn clean(&mut self) -> &mut Self{
        self.blob = self.commands.spawn((
            Blob,
            Visibility::Visible,
            ComputedVisibility::HIDDEN,
            TransformBundle::IDENTITY
        )).id();
        self.blocks = Vec::new();
        self.current_pos = None;
        self
    }

    /// move one step left from the current position
    pub fn left(&mut self) -> &mut Self{
        if self.current_pos.is_some(){
            let pos = self.current_pos.unwrap();
            if self.blocks[pos].left.is_some(){
                let index = self.blocks[pos].left.unwrap();
                self.current_pos = Some(index);
                return self;
            }
        }
        warn!("trying to reach a non-exist, return orginal block");
        self
    }

    /// move one step right from the current position
    pub fn right(&mut self) -> &mut Self{
        if self.current_pos.is_some(){
            let pos = self.current_pos.unwrap();
            if self.blocks[pos].right.is_some(){
                let index = self.blocks[pos].right.unwrap();
                self.current_pos = Some(index);
                return self;
            }
        };
        warn!("trying to reach a non-exist, return orginal block");
        self
    }

    /// move one step up from the current position
    pub fn top(&mut self) -> &mut Self{
        if self.current_pos.is_some(){
            let pos = self.current_pos.unwrap();
            if self.blocks[pos].top.is_some(){
                let index = self.blocks[pos].top.unwrap();
                self.current_pos = Some(index);
                return self;
            }
        };
        warn!("trying to reach a non-exist, return orginal block");
        self
    }

    /// move one step down from the current position
    pub fn bottom(&mut self) -> &mut Self{
        if self.current_pos.is_some(){
            let pos = self.current_pos.unwrap();
            if self.blocks[pos].bottom.is_some(){
                let index = self.blocks[pos].bottom.unwrap();
                self.current_pos = Some(index);
                return self;
            }
        };
        warn!("trying to reach a non-exist, return orginal block");
        self
    }

    /// reset the current position to the first block
    pub fn reset(&mut self) -> &mut Self{
        if self.current_pos.is_some(){
            self.current_pos = Some(0);
            return self;
        }
        warn!("trying to reset position for an empty BlobBuilder");
        self
    }

    /// create the first block and return itself
    pub fn create_first<T:Bundle>(
        &mut self,
        phy_block_bundle: PhysiBlockBundle,
        others: T) -> &mut Self{
        let id = self.commands.spawn(phy_block_bundle.clone().with_color(self.color)).insert(others).id();
        let block = BlobBlock{
            id: id,
            top: None,
            bottom: None,
            left: None,
            right: None,
            vec_index: 0,
            size: phy_block_bundle.sprite.sprite.custom_size.unwrap()/2.0,
            translation: phy_block_bundle.sprite.transform.translation.truncate(),
            anchors: phy_block_bundle.anchors
        };
        
        self.commands.entity(self.blob).push_children(&[block.id]);
        self.blocks.push(block);
        self.current_pos = Some(0);

        self
    }

    /// add a new block to the left of the current block and move the current position to that block
    pub fn add_to_left<T:Bundle>(
        &mut self, 
        dx:f32, 
        dy:f32, 
        motor_pos: Option<f32>, 
        motor_limits: Option<[f32; 2]>, 
        others: T) -> &mut Self{
        if self.current_pos.is_none(){
            warn!("trying to add a block while no parent block exist");
            return self;
        }
        let pos = self.current_pos.unwrap();
        let block = &mut self.blocks[pos];

        if block.left.is_some(){
            warn!("trying to add a block to an occupied position");
            return self;
        }

        let spawn_x = block.translation.x - block.size.x - dx;
        let spawn_y = block.translation.y;
        let phy_block_bundle = PhysiBlockBundle::from_xy_dx_dy(
            spawn_x, spawn_y, dx, dy
        ).with_color(self.color).with_density(DEFAULT_DENSITY);
        let id = self.commands.spawn(phy_block_bundle.clone()).insert(others).id();
        let new_block = BlobBlock{
            id: id,
            top: None,
            bottom: None,
            left: None,
            right: Some(pos),
            vec_index: self.blocks.len(),
            size: phy_block_bundle.sprite.sprite.custom_size.unwrap()/2.0,
            translation: phy_block_bundle.sprite.transform.translation.truncate(),
            anchors: phy_block_bundle.anchors
        };
        
        let block = &mut self.blocks[pos];
        block.left = Some(new_block.vec_index);
        self.current_pos = Some(new_block.vec_index);

        // set joint motor
        let mut stiff = 0.0;
        let mut motor_target = 0.0;
        if motor_pos.is_some(){
            stiff = MOTOR_STIFFNESS;
            motor_target = motor_pos.unwrap();
        }

        // set joint limits
        let mut limits = [-PI,PI];
        if motor_limits.is_some(){
            limits = motor_limits.unwrap()
        }

        let joint = RevoluteJointBuilder::new()
            .local_anchor1(block.anchors.left)
            .local_anchor2(new_block.anchors.right)
            .motor_position(motor_target, stiff, MOTOR_DAMPING)
            .limits(limits);

        bind_joint(&mut self.commands, block.id, new_block.id, joint);

        self.commands.entity(self.blob).push_children(&[new_block.id]);
        self.blocks.push(new_block);

        self
    }


    /// add a new block to the right of the current block and move the current position to that block
    pub fn add_to_right<T:Bundle>(
        &mut self, 
        dx:f32, 
        dy:f32, 
        motor_pos: Option<f32>, 
        motor_limits: Option<[f32; 2]>, 
        others: T) -> &mut Self{
        if self.current_pos.is_none(){
            warn!("trying to add a block while no parent block exist");
            return self;
        }
        let pos = self.current_pos.unwrap();
        let block = &mut self.blocks[pos];

        if block.right.is_some(){
            warn!("trying to add a block to an occupied position");
            return self;
        }
        let spawn_x = block.translation.x + block.size.x + dx;
        let spawn_y = block.translation.y;
        let phy_block_bundle = PhysiBlockBundle::from_xy_dx_dy(
            spawn_x, spawn_y, dx, dy
        ).with_color(self.color).with_density(DEFAULT_DENSITY);
        let id = self.commands.spawn(phy_block_bundle.clone()).insert(others).id();
        let new_block = BlobBlock{
            id: id,
            top: None,
            bottom: None,
            left: Some(pos),
            right: None,
            vec_index: self.blocks.len(),
            size: phy_block_bundle.sprite.sprite.custom_size.unwrap()/2.0,
            translation: phy_block_bundle.sprite.transform.translation.truncate(),
            anchors: phy_block_bundle.anchors
        };
        
        let block = &mut self.blocks[pos];
        block.right = Some(new_block.vec_index);
        self.current_pos = Some(new_block.vec_index);

        // set joint motor
        let mut stiff = 0.0;
        let mut motor_target = 0.0;
        if motor_pos.is_some(){
            stiff = MOTOR_STIFFNESS;
            motor_target = motor_pos.unwrap();
        }

        // set joint limits
        let mut limits = [-PI,PI];
        if motor_limits.is_some(){
            limits = motor_limits.unwrap()
        }

        let joint = RevoluteJointBuilder::new()
            .local_anchor1(block.anchors.right)
            .local_anchor2(new_block.anchors.left)
            .motor_position(motor_target, stiff, MOTOR_DAMPING)
            .limits(limits);

        bind_joint(&mut self.commands, block.id, new_block.id, joint);

        self.commands.entity(self.blob).push_children(&[new_block.id]);
        self.blocks.push(new_block);

        self
    }


    /// add a new block to the top of the current block and move the current position to that block
    pub fn add_to_top<T:Bundle>(
        &mut self, 
        dx:f32, 
        dy:f32, 
        motor_pos: Option<f32>, 
        motor_limits: Option<[f32; 2]>, 
        others: T) -> &mut Self{
        if self.current_pos.is_none(){
            warn!("trying to add a block while no parent block exist");
            return self;
        }
        let pos = self.current_pos.unwrap();
        let block = &mut self.blocks[pos];

        if block.top.is_some(){
            warn!("trying to add a block to an occupied position");
            return self;
        }

        let spawn_x = block.translation.x;
        let spawn_y = block.translation.y + block.size.y + dy;
        let phy_block_bundle = PhysiBlockBundle::from_xy_dx_dy(
            spawn_x, spawn_y, dx, dy
        ).with_color(self.color).with_density(DEFAULT_DENSITY);
        let id = self.commands.spawn(phy_block_bundle.clone()).insert(others).id();
        let new_block = BlobBlock{
            id: id,
            top: None,
            bottom: Some(pos),
            left: None,
            right: None,
            vec_index: self.blocks.len(),
            size: phy_block_bundle.sprite.sprite.custom_size.unwrap()/2.0,
            translation: phy_block_bundle.sprite.transform.translation.truncate(),
            anchors: phy_block_bundle.anchors
        };
        
        let block = &mut self.blocks[pos];
        block.top = Some(new_block.vec_index);
        self.current_pos = Some(new_block.vec_index);

        // set joint motor
        let mut stiff = 0.0;
        let mut motor_target = 0.0;
        if motor_pos.is_some(){
            stiff = MOTOR_STIFFNESS;
            motor_target = motor_pos.unwrap();
        }

        // set joint limits
        let mut limits = [-PI,PI];
        if motor_limits.is_some(){
            limits = motor_limits.unwrap()
        }

        let joint = RevoluteJointBuilder::new()
            .local_anchor1(block.anchors.top)
            .local_anchor2(new_block.anchors.bottom)
            .motor_position(motor_target, stiff, MOTOR_DAMPING)
            .limits(limits);

        bind_joint(&mut self.commands, block.id, new_block.id, joint);

        self.commands.entity(self.blob).push_children(&[new_block.id]);
        self.blocks.push(new_block);

        self
    }


    /// add a new block to the bottom of the current block and move the current position to that block
    pub fn add_to_bottom<T:Bundle>(
        &mut self, 
        dx:f32, 
        dy:f32, 
        motor_pos: Option<f32>, 
        motor_limits: Option<[f32; 2]>, 
        others: T) -> &mut Self{
        if self.current_pos.is_none(){
            warn!("trying to add a block while no parent block exist");
            return self;
        }
        let pos = self.current_pos.unwrap();
        let block = &mut self.blocks[pos];

        if block.bottom.is_some(){
            warn!("trying to add a block to an occupied position");
            return self;
        }

        let spawn_x = block.translation.x;
        let spawn_y = block.translation.y - block.size.y - dy;
        let phy_block_bundle = PhysiBlockBundle::from_xy_dx_dy(
            spawn_x, spawn_y, dx, dy
        ).with_color(self.color).with_density(DEFAULT_DENSITY);
        let id = self.commands.spawn(phy_block_bundle.clone()).insert(others).id();
        let new_block = BlobBlock{
            id: id,
            top: Some(pos),
            bottom: None,
            left: None,
            right: None,
            vec_index: self.blocks.len(),
            size: phy_block_bundle.sprite.sprite.custom_size.unwrap()/2.0,
            translation: phy_block_bundle.sprite.transform.translation.truncate(),
            anchors: phy_block_bundle.anchors
        };
        
        let block = &mut self.blocks[pos];
        block.bottom = Some(new_block.vec_index);
        self.current_pos = Some(new_block.vec_index);

        // set joint motor
        let mut stiff = 0.0;
        let mut motor_target = 0.0;
        if motor_pos.is_some(){
            stiff = MOTOR_STIFFNESS;
            motor_target = motor_pos.unwrap();
        }

        // set joint limits
        let mut limits = [-PI,PI];
        if motor_limits.is_some(){
            limits = motor_limits.unwrap()
        }

        let joint = RevoluteJointBuilder::new()
            .local_anchor1(block.anchors.bottom)
            .local_anchor2(new_block.anchors.top)
            .motor_position(motor_target, stiff, MOTOR_DAMPING)
            .limits(limits);

        bind_joint(&mut self.commands, block.id, new_block.id, joint);

        self.commands.entity(self.blob).push_children(&[new_block.id]);
        self.blocks.push(new_block);

        self
    }
    
}

// helper function
pub fn bind_joint(
    commands: &mut Commands,
    parent: Entity,
    child: Entity,
    joint: RevoluteJointBuilder,
){
    commands.entity(child).with_children(|cmd| {
        let mut new_joint = ImpulseJoint::new(parent, joint);
        new_joint.data.set_contacts_enabled(ENABLE_CONTACTS);
        cmd.spawn(new_joint);
    });
}