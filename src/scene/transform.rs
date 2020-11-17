//! Contains all structures and methods to create and manage transforms.
//!
//! Transform allows you to combine spatial properties into single matrix in
//! easy manner. It contains many methods that can be used to modify a single
//! property of transform which then will be "baked" into single matrix.
//!
//! # Complexity
//!
//! rg3d uses complex transform model inherited from FBX transform formulae:
//!
//! http://download.autodesk.com/us/fbx/20112/FBX_SDK_HELP/index.html?url=WS1a9193826455f5ff1f92379812724681e696651.htm,topicNumber=d0e7429
//!
//! Transform = T * Roff * Rp * Rpre * R * Rpost * Rp⁻¹ * Soff * Sp * S * Sp⁻¹
//!
//! where
//! T     - Translation
//! Roff  - Rotation offset
//! Rp    - Rotation pivot
//! Rpre  - Pre-rotation
//! R     - Rotation
//! Rpost - Post-rotation
//! Rp⁻¹  - Inverse of the rotation pivot
//! Soff  - Scaling offset
//! Sp    - Scaling pivot
//! S     - Scaling
//! Sp⁻¹  - Inverse of the scaling pivot
//!
//! It is very flexible, however it can be slow in computation. To solve possible
//! performance issues, rg3d tries to precache every possible component. This means
//! that we use lazy evaluation: you can setup all required properties, and actual
//! calculations will be delayed until you try to get matrix from transform. This makes
//! calculations faster, but increases required amount of memory.
//!
//! In most cases you don't need to bother about all those properties, you need just T R S -
//! it will cover 99% of requirements.
//!
//! Fun fact: transform format was dictated by the use of monster called FBX file format.
//! Some libraries (like assimp) decomposes this complex formula into set of smaller transforms
//! which are contains only T R S components and then combine them to get final result, I find
//! this approach very bug prone, and it is still heavy from computation side. It is much
//! easier to uses it as is.
//!
//! # Decomposition
//!
//! Once transform baked into matrix, it is *almost* impossible to decompose it back into
//! initial components, thats why engine does not provide any methods to get those
//! properties back.

use crate::{
    core::{
        algebra::{Matrix3, Matrix4, UnitQuaternion, Vector3},
        visitor::{Visit, VisitResult, Visitor},
    },
    utils::log::Log,
};
use std::cell::Cell;

/// See module docs.
#[derive(Clone, Debug)]
pub struct Transform {
    /// Indicates that some property has changed and matrix must be
    /// recalculated before use. This is some sort of lazy evaluation.
    dirty: Cell<bool>,
    local_scale: Vector3<f32>,
    local_position: Vector3<f32>,
    local_rotation: UnitQuaternion<f32>,
    pre_rotation: UnitQuaternion<f32>,
    post_rotation: UnitQuaternion<f32>,
    rotation_offset: Vector3<f32>,
    rotation_pivot: Vector3<f32>,
    scaling_offset: Vector3<f32>,
    scaling_pivot: Vector3<f32>,
    /// Combined transform. Final result of combination of other properties.
    matrix: Cell<Matrix4<f32>>,
    post_rotation_matrix: Matrix3<f32>,
}

impl Visit for Transform {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.local_scale.visit("LocalScale", visitor)?;
        self.local_position.visit("LocalPosition", visitor)?;
        self.local_rotation.visit("LocalRotation", visitor)?;
        self.pre_rotation.visit("PreRotation", visitor)?;
        self.post_rotation.visit("PostRotation", visitor)?;
        self.rotation_offset.visit("RotationOffset", visitor)?;
        self.rotation_pivot.visit("RotationPivot", visitor)?;
        self.scaling_offset.visit("ScalingOffset", visitor)?;
        self.scaling_pivot.visit("ScalingPivot", visitor)?;

        visitor.leave_region()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// Creates new transform that has no effect, in other words any vector
    /// or matrix will remain unchanged if combined with identity transform.
    pub fn identity() -> Self {
        Self {
            dirty: Cell::new(true),
            local_position: Vector3::default(),
            local_scale: Vector3::new(1.0, 1.0, 1.0),
            local_rotation: UnitQuaternion::identity(),
            pre_rotation: UnitQuaternion::identity(),
            post_rotation: UnitQuaternion::identity(),
            rotation_offset: Vector3::default(),
            rotation_pivot: Vector3::default(),
            scaling_offset: Vector3::default(),
            scaling_pivot: Vector3::default(),
            matrix: Cell::new(Matrix4::identity()),
            post_rotation_matrix: Matrix3::identity(),
        }
    }

    /// Returns current position of transform.
    #[inline]
    pub fn position(&self) -> Vector3<f32> {
        self.local_position
    }

    /// Sets position of transform.
    #[inline]
    pub fn set_position(&mut self, local_position: Vector3<f32>) -> &mut Self {
        if self.dirty.get() || self.local_position != local_position {
            self.local_position = local_position;
            self.dirty.set(true);
        }
        self
    }

    /// Returns current rotation quaternion of transform.
    #[inline]
    pub fn rotation(&self) -> UnitQuaternion<f32> {
        self.local_rotation
    }

    /// Sets rotation of transform.
    #[inline]
    pub fn set_rotation(&mut self, local_rotation: UnitQuaternion<f32>) -> &mut Self {
        if self.dirty.get() || self.local_rotation != local_rotation {
            self.local_rotation = local_rotation;
            self.dirty.set(true);
        }
        self
    }

    /// Returns current scale factor of transform.
    #[inline]
    pub fn scale(&self) -> Vector3<f32> {
        self.local_scale
    }

    /// Sets scale of transform. It is strongly advised to use only uniform scaling,
    /// non-uniform is possible but it can lead to very "interesting" effects, also
    /// non-uniform scaling possible will be removed in future, especially if engine
    /// migrate to some full-featured physics engine.
    #[inline]
    pub fn set_scale(&mut self, local_scale: Vector3<f32>) -> &mut Self {
        if self.dirty.get() || self.local_scale != local_scale {
            self.local_scale = local_scale;
            self.dirty.set(true);
        }
        self
    }

    /// Sets pre-rotation of transform. Usually pre-rotation can be used to change
    /// "coordinate" system of transform. It is mostly for FBX compatibility, and
    /// never used in other places of engine.
    #[inline]
    pub fn set_pre_rotation(&mut self, pre_rotation: UnitQuaternion<f32>) -> &mut Self {
        if self.dirty.get() || self.pre_rotation != pre_rotation {
            self.pre_rotation = pre_rotation;
            self.dirty.set(true);
        }
        self
    }

    /// Returns current pre-rotation of transform.
    #[inline]
    pub fn pre_rotation(&self) -> UnitQuaternion<f32> {
        self.pre_rotation
    }

    /// Sets post-rotation of transform. Usually post-rotation can be used to change
    /// "coordinate" system of transform. It is mostly for FBX compatibility, and
    /// never used in other places of engine.
    #[inline]
    pub fn set_post_rotation(&mut self, post_rotation: UnitQuaternion<f32>) -> &mut Self {
        if self.dirty.get() || self.post_rotation != post_rotation {
            self.post_rotation = post_rotation;
            self.post_rotation_matrix = self
                .post_rotation
                .to_rotation_matrix()
                .matrix()
                .try_inverse()
                .unwrap_or_else(|| {
                    Log::writeln(
                        "Unable to inverse post rotation matrix! Fallback to identity matrix."
                            .to_owned(),
                    );
                    Matrix3::identity()
                });
            self.dirty.set(true);
        }
        self
    }

    /// Returns current post-rotation of transform.
    #[inline]
    pub fn post_rotation(&self) -> UnitQuaternion<f32> {
        self.post_rotation
    }

    /// Sets rotation offset of transform. Moves rotation pivot using given vector,
    /// it results in rotation being performed around rotation pivot with some offset.
    #[inline]
    pub fn set_rotation_offset(&mut self, rotation_offset: Vector3<f32>) -> &mut Self {
        if self.dirty.get() || self.rotation_offset != rotation_offset {
            self.rotation_offset = rotation_offset;
            self.dirty.set(true);
        }
        self
    }

    /// Returns current rotation offset of transform.
    #[inline]
    pub fn rotation_offset(&self) -> Vector3<f32> {
        self.rotation_offset
    }

    /// Sets rotation pivot of transform. This method sets a point around which all
    /// rotations will be performed. For example it can be used to rotate a cube around
    /// its vertex.
    #[inline]
    pub fn set_rotation_pivot(&mut self, rotation_pivot: Vector3<f32>) -> &mut Self {
        if self.dirty.get() || self.rotation_pivot != rotation_pivot {
            self.rotation_pivot = rotation_pivot;
            self.dirty.set(true);
        }
        self
    }

    /// Returns current rotation pivot of transform.
    #[inline]
    pub fn rotation_pivot(&self) -> Vector3<f32> {
        self.rotation_pivot
    }

    /// Sets scaling offset. Scaling offset defines offset from position of scaling
    /// pivot.
    #[inline]
    pub fn set_scaling_offset(&mut self, scaling_offset: Vector3<f32>) -> &mut Self {
        if self.dirty.get() || self.scaling_offset != scaling_offset {
            self.scaling_offset = scaling_offset;
            self.dirty.set(true);
        }
        self
    }

    /// Returns current scaling offset of transform.
    #[inline]
    pub fn scaling_offset(&self) -> Vector3<f32> {
        self.scaling_offset
    }

    /// Sets scaling pivot. Scaling pivot sets a point around which scale will be
    /// performed.
    #[inline]
    pub fn set_scaling_pivot(&mut self, scaling_pivot: Vector3<f32>) -> &mut Self {
        if self.dirty.get() || self.scaling_pivot != scaling_pivot {
            self.scaling_pivot = scaling_pivot;
            self.dirty.set(true);
        }
        self
    }

    /// Returns current scaling pivot of transform.
    #[inline]
    pub fn scaling_pivot(&self) -> Vector3<f32> {
        self.scaling_pivot
    }

    /// Shifts local position using given vector. It is a shortcut for:
    /// set_position(position() + offset)
    #[inline]
    pub fn offset(&mut self, vec: Vector3<f32>) -> &mut Self {
        self.local_position += vec;
        self.dirty.set(true);
        self
    }

    fn calculate_local_transform(&self) -> Matrix4<f32> {
        // Make shortcuts to remove visual clutter.
        let por = &self.post_rotation_matrix;
        let pr = *self.pre_rotation.to_rotation_matrix().matrix();
        let r = *self.local_rotation.to_rotation_matrix().matrix();

        let sx = self.local_scale.x;
        let sy = self.local_scale.y;
        let sz = self.local_scale.z;

        let tx = self.local_position.x;
        let ty = self.local_position.y;
        let tz = self.local_position.z;

        let rpx = self.rotation_pivot.x;
        let rpy = self.rotation_pivot.y;
        let rpz = self.rotation_pivot.z;

        let rox = self.rotation_offset.x;
        let roy = self.rotation_offset.y;
        let roz = self.rotation_offset.z;

        let spx = self.scaling_pivot.x;
        let spy = self.scaling_pivot.y;
        let spz = self.scaling_pivot.z;

        let sox = self.scaling_offset.x;
        let soy = self.scaling_offset.y;
        let soz = self.scaling_offset.z;

        // Optimized multiplication of these matrices:
        //
        // Transform = T * Roff * Rp * Rpre * R * Rpost * Rp⁻¹ * Soff * Sp * S * Sp⁻¹
        //
        // where
        // T     - Translation
        // Roff  - Rotation offset
        // Rp    - Rotation pivot
        // Rpre  - Pre-rotation
        // R     - Rotation
        // Rpost - Post-rotation
        // Rp⁻¹  - Inverse of the rotation pivot
        // Soff  - Scaling offset
        // Sp    - Scaling pivot
        // S     - Scaling
        // Sp⁻¹  - Inverse of the scaling pivot
        let a0 = pr[0] * r[0] + pr[3] * r[1] + pr[6] * r[2];
        let a1 = pr[1] * r[0] + pr[4] * r[1] + pr[7] * r[2];
        let a2 = pr[2] * r[0] + pr[5] * r[1] + pr[8] * r[2];
        let a3 = pr[0] * r[3] + pr[3] * r[4] + pr[6] * r[5];
        let a4 = pr[1] * r[3] + pr[4] * r[4] + pr[7] * r[5];
        let a5 = pr[2] * r[3] + pr[5] * r[4] + pr[8] * r[5];
        let a6 = pr[0] * r[6] + pr[3] * r[7] + pr[6] * r[8];
        let a7 = pr[1] * r[6] + pr[4] * r[7] + pr[7] * r[8];
        let a8 = pr[2] * r[6] + pr[5] * r[7] + pr[8] * r[8];
        let f0 = por[0] * a0 + por[1] * a3 + por[2] * a6;
        let f1 = por[0] * a1 + por[1] * a4 + por[2] * a7;
        let f2 = por[0] * a2 + por[1] * a5 + por[2] * a8;
        let f3 = por[3] * a0 + por[4] * a3 + por[5] * a6;
        let f4 = por[3] * a1 + por[4] * a4 + por[5] * a7;
        let f5 = por[3] * a2 + por[4] * a5 + por[5] * a8;
        let f6 = por[6] * a0 + por[7] * a3 + por[8] * a6;
        let f7 = por[6] * a1 + por[7] * a4 + por[8] * a7;
        let f8 = por[6] * a2 + por[7] * a5 + por[8] * a8;
        let m0 = sx * f0;
        let m1 = sx * f1;
        let m2 = sx * f2;
        let m3 = 0.0;
        let m4 = sy * f3;
        let m5 = sy * f4;
        let m6 = sy * f5;
        let m7 = 0.0;
        let m8 = sz * f6;
        let m9 = sz * f7;
        let m10 = sz * f8;
        let m11 = 0.0;
        let k0 = spx * f0;
        let k1 = spy * f3;
        let k2 = spz * f6;
        let m12 = rox + rpx + tx - rpx * f0 - rpy * f3 - rpz * f6
            + sox * f0
            + k0
            + soy * f3
            + k1
            + soz * f6
            + k2
            - sx * k0
            - sy * k1
            - sz * k2;
        let k3 = spx * f1;
        let k4 = spy * f4;
        let k5 = spz * f7;
        let m13 = roy + rpy + ty - rpx * f1 - rpy * f4 - rpz * f7
            + sox * f1
            + k3
            + soy * f4
            + k4
            + soz * f7
            + k5
            - sx * k3
            - sy * k4
            - sz * k5;
        let k6 = spx * f2;
        let k7 = spy * f5;
        let k8 = spz * f8;
        let m14 = roz + rpz + tz - rpx * f2 - rpy * f5 - rpz * f8
            + sox * f2
            + k6
            + soy * f5
            + k7
            + soz * f8
            + k8
            - sx * k6
            - sy * k7
            - sz * k8;
        let m15 = 1.0;
        Matrix4::new(
            m0, m4, m8, m12, m1, m5, m9, m13, m2, m6, m10, m14, m3, m7, m11, m15,
        )
    }

    /// Returns matrix which is final result of transform. Matrix then can be used to transform
    /// a vector, or combine with other matrix, to make transform hierarchy for example.
    pub fn matrix(&self) -> Matrix4<f32> {
        if self.dirty.get() {
            self.matrix.set(self.calculate_local_transform());
            self.dirty.set(false)
        }
        self.matrix.get()
    }
}

/// Transform builder allows you to construct transform in declarative manner.
/// This is typical implementation of Builder pattern.
pub struct TransformBuilder {
    local_scale: Option<Vector3<f32>>,
    local_position: Option<Vector3<f32>>,
    local_rotation: Option<UnitQuaternion<f32>>,
    pre_rotation: Option<UnitQuaternion<f32>>,
    post_rotation: Option<UnitQuaternion<f32>>,
    rotation_offset: Option<Vector3<f32>>,
    rotation_pivot: Option<Vector3<f32>>,
    scaling_offset: Option<Vector3<f32>>,
    scaling_pivot: Option<Vector3<f32>>,
}

impl Default for TransformBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformBuilder {
    /// Creates new transform builder. If it won't be modified then it will produce
    /// identity transform as result.
    pub fn new() -> Self {
        Self {
            local_scale: None,
            local_position: None,
            local_rotation: None,
            pre_rotation: None,
            post_rotation: None,
            rotation_offset: None,
            rotation_pivot: None,
            scaling_offset: None,
            scaling_pivot: None,
        }
    }

    /// Sets desired local scale.
    pub fn with_local_scale(mut self, scale: Vector3<f32>) -> Self {
        self.local_scale = Some(scale);
        self
    }

    /// Sets desired local position.
    pub fn with_local_position(mut self, position: Vector3<f32>) -> Self {
        self.local_position = Some(position);
        self
    }

    /// Sets desired local rotation.
    pub fn with_local_rotation(mut self, rotation: UnitQuaternion<f32>) -> Self {
        self.local_rotation = Some(rotation);
        self
    }

    /// Sets desired pre-rotation.
    pub fn with_pre_rotation(mut self, rotation: UnitQuaternion<f32>) -> Self {
        self.pre_rotation = Some(rotation);
        self
    }

    /// Sets desired post-rotation.
    pub fn with_post_rotation(mut self, rotation: UnitQuaternion<f32>) -> Self {
        self.post_rotation = Some(rotation);
        self
    }

    /// Sets desired rotation offset.
    pub fn with_rotation_offset(mut self, offset: Vector3<f32>) -> Self {
        self.rotation_offset = Some(offset);
        self
    }

    /// Sets desired rotation pivot.
    pub fn with_rotation_pivot(mut self, pivot: Vector3<f32>) -> Self {
        self.rotation_pivot = Some(pivot);
        self
    }

    /// Sets desired scaling offset.
    pub fn with_scaling_offset(mut self, offset: Vector3<f32>) -> Self {
        self.scaling_offset = Some(offset);
        self
    }

    /// Sets desired scaling pivot.
    pub fn with_scaling_pivot(mut self, pivot: Vector3<f32>) -> Self {
        self.scaling_pivot = Some(pivot);
        self
    }

    /// Builds new Transform instance using provided values.
    pub fn build(self) -> Transform {
        Transform {
            dirty: Cell::new(true),
            local_scale: self
                .local_scale
                .unwrap_or_else(|| Vector3::new(1.0, 1.0, 1.0)),
            local_position: self.local_position.unwrap_or_default(),
            local_rotation: self.local_rotation.unwrap_or_else(UnitQuaternion::identity),
            pre_rotation: self.pre_rotation.unwrap_or_else(UnitQuaternion::identity),
            post_rotation: self.post_rotation.unwrap_or_else(UnitQuaternion::identity),
            rotation_offset: self.rotation_offset.unwrap_or_default(),
            rotation_pivot: self.rotation_pivot.unwrap_or_default(),
            scaling_offset: self.scaling_offset.unwrap_or_default(),
            scaling_pivot: self.scaling_pivot.unwrap_or_default(),
            matrix: Cell::new(Matrix4::identity()),
            post_rotation_matrix: Matrix3::identity(),
        }
    }
}
