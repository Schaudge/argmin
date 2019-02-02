// Copyright 2018 Stefan Kroboth
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! # References:
//!
//! [0] Jorge Nocedal and Stephen J. Wright (2006). Numerical Optimization.
//! Springer. ISBN 0-387-30303-0.

use crate::prelude::*;
use std;
use std::default::Default;

/// Newton's method iteratively finds the stationary points of a function f by using a second order
/// approximation of f at the current point.
///
/// # Example
///
/// ```
/// # extern crate argmin;
/// # extern crate ndarray;
/// use argmin::prelude::*;
/// use argmin::solver::newton::Newton;
/// # use argmin::testfunctions::{rosenbrock_2d, rosenbrock_2d_derivative, rosenbrock_2d_hessian};
/// use ndarray::{Array, Array1, Array2};
///
/// # #[derive(Clone)]
/// # struct MyProblem {}
/// #
/// # impl ArgminOperator for MyProblem {
/// #     type Parameters = Array1<f64>;
/// #     type OperatorOutput = f64;
/// #     type Hessian = Array2<f64>;
/// #
/// #     fn apply(&self, p: &Self::Parameters) -> Result<Self::OperatorOutput, Error> {
/// #         Ok(rosenbrock_2d(&p.to_vec(), 1.0, 100.0))
/// #     }
/// #
/// #     fn gradient(&self, p: &Self::Parameters) -> Result<Self::Parameters, Error> {
/// #         Ok(Array1::from_vec(rosenbrock_2d_derivative(&p.to_vec(), 1.0, 100.0)))
/// #     }
/// #
/// #     fn hessian(&self, p: &Self::Parameters) -> Result<Self::Hessian, Error> {
/// #         let h = rosenbrock_2d_hessian(&p.to_vec(), 1.0, 100.0);
/// #         Ok(Array::from_shape_vec((2, 2), h)?)
/// #     }
/// # }
/// #
/// # fn run() -> Result<(), Error> {
/// // Define cost function
/// let cost = MyProblem {};
///
/// // Define initial parameter vector
/// let init_param: Array1<f64> = Array1::from_vec(vec![-1.2, 1.0]);
///
/// // Set up solver
/// let mut solver = Newton::new(&cost, init_param);
///
/// // Set maximum number of iterations
/// solver.set_max_iters(7);
///
/// // Attach a logger
/// solver.add_logger(ArgminSlogLogger::term());
///
/// // Run solver
/// solver.run()?;
///
/// // Wait a second (lets the logger flush everything before printing again)
/// std::thread::sleep(std::time::Duration::from_secs(1));
///
/// // Print result
/// println!("{:?}", solver.result());
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     if let Err(ref e) = run() {
/// #         println!("{} {}", e.as_fail(), e.backtrace());
/// #         std::process::exit(1);
/// #     }
/// # }
/// ```
///
/// # References:
///
/// [0] Jorge Nocedal and Stephen J. Wright (2006). Numerical Optimization.
/// Springer. ISBN 0-387-30303-0.
#[derive(ArgminSolver)]
pub struct Newton<'a, T, H>
where
    T: 'a + Clone + Default + ArgminScaledSub<T, f64, T>,
    H: 'a + Clone + Default + ArgminInv<H> + ArgminDot<T, T>,
{
    /// gamma
    gamma: f64,
    /// Base stuff
    base: ArgminBase<'a, T, f64, H>,
}

impl<'a, T, H> Newton<'a, T, H>
where
    T: 'a + Clone + Default + ArgminScaledSub<T, f64, T>,
    H: 'a + Clone + Default + ArgminInv<H> + ArgminDot<T, T>,
{
    /// Constructor
    pub fn new(
        cost_function: &'a ArgminOperator<Parameters = T, OperatorOutput = f64, Hessian = H>,
        init_param: T,
    ) -> Self {
        Newton {
            gamma: 1.0,
            base: ArgminBase::new(cost_function, init_param),
        }
    }

    /// set gamma
    pub fn set_gamma(&mut self, gamma: f64) -> Result<&mut Self, Error> {
        if gamma <= 0.0 || gamma > 1.0 {
            return Err(ArgminError::InvalidParameter {
                text: "Newton: gamma must be in  (0, 1].".to_string(),
            }
            .into());
        }
        self.gamma = gamma;
        Ok(self)
    }
}

impl<'a, T, H> ArgminNextIter for Newton<'a, T, H>
where
    T: 'a + Clone + Default + ArgminScaledSub<T, f64, T>,
    H: 'a + Clone + Default + ArgminInv<H> + ArgminDot<T, T>,
{
    type Parameters = T;
    type OperatorOutput = f64;
    type Hessian = H;

    fn next_iter(&mut self) -> Result<ArgminIterationData<Self::Parameters>, Error> {
        let param = self.cur_param();
        let grad = self.gradient(&param)?;
        let hessian = self.hessian(&param)?;
        let new_param = param.scaled_sub(&self.gamma, &hessian.ainv()?.dot(&grad));
        let out = ArgminIterationData::new(new_param, 0.0);
        Ok(out)
    }
}
