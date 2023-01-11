// Copyright (c) 2022, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use nalgebra as na;

pub use na::Vector2 as Vec2;
pub use na::Vector3 as Vec3;
pub use na::Vector4 as Vec4;

pub type Vec2f = Vec2<f64>;
pub type Vec3f = Vec3<f64>;
pub type Vec4f = Vec4<f64>;

pub trait Clamp {
    fn clamp(&self, min: &Self, max: &Self) -> Self;
}

impl<T: Clone + PartialOrd, D: Clone + na::Dim, S: Clone + na::RawStorageMut<T, D, na::U1>> Clamp for na::Vector<T, D, S> {
    fn clamp(&self, min: &Self, max: &Self) -> Self {
        let mut new = self.clone();
        for i in 0..self.data.shape().0.value() {
            if new[i] < min[i] {
                new[i] = min[i].clone();
            } else if new[i] > max[i] {
                new[i] = max[i].clone();
            }
        }
        new
    }
}

impl<T: na::Scalar + Clone + PartialOrd, const D: usize> Clamp for na::Point<T, D> {
    fn clamp(&self, min: &Self, max: &Self) -> Self {
        self.coords.clamp(&min.coords, &max.coords).into()
    }
}

pub trait Gaussian2d {
    fn gaussian2d(self, sigma: Self) -> Self;
}

impl Gaussian2d for f32 {
    fn gaussian2d(self, sigma: Self) -> Self {
        let term1 = 1.0 / 2.0 * std::f32::consts::PI * (sigma * sigma);
        let term2 = (-(self / (2.0 * sigma * sigma))).exp();
        term1 * term2
    }
}

impl Gaussian2d for f64 {
    fn gaussian2d(self, sigma: Self) -> Self {
        let term1 = 1.0 / 2.0 * std::f64::consts::PI * (sigma * sigma);
        let term2 = (-(self / (2.0 * sigma * sigma))).exp();
        term1 * term2
    }
}
