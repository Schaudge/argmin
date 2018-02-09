// Copyright 2018 Stefan Kroboth
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Backtracking Line Search
//!
//! TODO

use std;
use errors::*;
use ndarray::{Array1, arr1};
use ArgminSolver;
use result::ArgminResult;
use termination::TerminationReason;

/// Backtracking Line Search
pub struct BacktrackingLineSearch<'a> {
    /// Reference to cost function.
    cost_function: &'a Fn(&Array1<f64>) -> f64,
    /// Gradient
    gradient: &'a Fn(&Array1<f64>) -> Array1<f64>,
    /// Starting distance to the current point:
    alpha: f64,
    /// Maximum number of iterations
    max_iters: u64,
    /// Parameter `tau`
    tau: f64,
    /// Parameter `c`
    c: f64,
    /// Current state
    state: Option<BacktrackingLineSearchState>,
}

/// Current state of the backtracking line search algorithm
pub struct BacktrackingLineSearchState {
    /// Search direction
    p: Array1<f64>,
    /// Starting point
    x: Array1<f64>,
    /// Current cost value
    cost: f64,
    /// t (TODO)
    t: f64,
    /// Cost function value at starting point
    fx: f64,
    /// Current number of iteration
    iter: u64,
    /// Current alpha
    alpha: f64,
}

impl<'a> BacktrackingLineSearch<'a> {
    /// Initialize Backtracking Line Search
    ///
    /// Requires the cost function and gradient to be passed as parameter. The parameters
    /// `max_iters`, `tau`, and `c` are set to 100, 0.5 and 0.5, respectively.
    pub fn new(
        cost_function: &'a Fn(&Array1<f64>) -> f64,
        gradient: &'a Fn(&Array1<f64>) -> Array1<f64>,
    ) -> Self {
        BacktrackingLineSearch {
            cost_function: cost_function,
            gradient: gradient,
            alpha: 1.0,
            max_iters: 100,
            tau: 0.5,
            c: 0.5,
            state: None,
        }
    }

    /// Set the maximum distance from the starting point
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = alpha;
        self
    }

    /// Set the maximum number of iterations
    pub fn max_iters(&mut self, max_iters: u64) -> &mut Self {
        self.max_iters = max_iters;
        self
    }

    /// Set c to a desired value between 0 and 1
    pub fn c(&mut self, c: f64) -> Result<&mut Self> {
        if c >= 1.0 || c <= 0.0 {
            return Err(ErrorKind::InvalidParameter(
                "BacktrackingLineSearch: Parameter `c` must satisfy 0 < c < 1.".into(),
            ).into());
        }
        self.c = c;
        Ok(self)
    }

    /// Set tau to a desired value between 0 and 1
    pub fn tau(&mut self, tau: f64) -> Result<&mut Self> {
        if tau >= 1.0 || tau <= 0.0 {
            return Err(ErrorKind::InvalidParameter(
                "BacktrackingLineSearch: Parameter `tau` must satisfy 0 < tau < 1.".into(),
            ).into());
        }
        self.tau = tau;
        Ok(self)
    }
}

impl<'a> ArgminSolver<'a> for BacktrackingLineSearch<'a> {
    type Parameter = Array1<f64>;
    type CostValue = f64;
    type Hessian = Array1<f64>;
    type StartingPoints = Array1<f64>;
    type ProblemDefinition = Array1<f64>;

    fn init(&mut self, p: Self::ProblemDefinition, x: &Self::StartingPoints) -> Result<()> {
        let m: f64 = p.t().dot(&((self.gradient)(x)));
        self.state = Some(BacktrackingLineSearchState {
            cost: std::f64::NAN,
            p: p,
            x: x.to_owned(),
            t: -self.c * m,
            fx: (self.cost_function)(x),
            iter: 0,
            alpha: self.alpha,
        });
        Ok(())
    }

    fn next_iter(&mut self) -> Result<ArgminResult<Self::Parameter, Self::CostValue>> {
        let mut state = self.state.take().unwrap();
        let param = &state.x + &(state.alpha * &state.p);
        state.cost = (self.cost_function)(&param);
        state.iter += 1;
        state.alpha *= self.tau;
        let mut out = ArgminResult::new(arr1(&[state.alpha]), std::f64::NAN, state.iter);
        self.state = Some(state);
        out.set_termination_reason(self.terminate());
        Ok(out)
    }

    /// Indicates whether any of the stopping criteria are met
    make_terminate!(self,
        self.state.as_ref().unwrap().iter >= self.max_iters, TerminationReason::MaxItersReached;
        self.state.as_ref().unwrap().fx - self.state.as_ref().unwrap().cost >= self.state.as_ref().unwrap().alpha * self.state.as_ref().unwrap().t, TerminationReason::TargetCostReached; 
    );

    /// Run gradient descent method
    make_run!(
        Self::ProblemDefinition,
        Self::StartingPoints,
        Self::Parameter,
        Self::CostValue
    );
}
