// timestep
pub const RAPIER_DT:f32 = 1.0/60.0;
pub const RAPIER_SUBSTEPS:usize = 1;

// scale world size
pub const WORLD_WIDTH:f32 = 10000.0;
pub const WORLD_HEIGHT:f32 = 10000.0;

// joint config
pub const MOTOR_STIFFNESS:f32 = 10.0;
pub const MOTOR_DAMPING:f32 = 0.0;
pub const ENABLE_CONTACTS:bool = false;
// joint contorl
pub const MOTOR_MAX_TARGET_V:f32 = 10.0;

// math
pub const EPSILON:f32 = 0.0001; // max error

// physics
pub const DRAG_COEFF:f32 = 5.0; // drag coefficient in fluid simulation
pub const DEFAULT_DENSITY:f32 = 1.0;

// Geno
pub const GENO_MAX_DEPTH:u32 = 2; // max recursion depth of Geno type
pub const DEFAULT_BLOCK_SIZE:[f32;2] = [50.0,50.0];

// Rand
pub const RAND_NODE_NOT_NONE:f64 = 0.9;
pub const RAND_SIZE_SCALER:[f32;2] = [0.5,2.0];

// nn
/// each children has 4 input values
/// 
/// shape is for nalgebra's matrix
pub const INWARD_NN_CHILDREN_INPUT_LEN:usize = 4;