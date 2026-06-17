// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// wasm-lambda
// src/interpreter.rs
//
// Makabaka1880, 2026. All rights reserved.

use std::rc::Rc;
use serde::Serialize;

use crate::machine::*;

#[derive(Serialize)]
pub struct KrivMachine {
    term: Rc<Term>,
    env: Rc<Environment>,
    stack: Rc<Stack>,
}

impl KrivMachine {
    pub fn new(term: Term) -> Self {
        KrivMachine {
            term: Rc::new(term),
            env: Rc::new(Environment::Empty),
            stack: Rc::new(Stack::Empty),
        }
    }

    pub fn step(&mut self) -> Result<bool, String> {
        let term = Rc::clone(&self.term);
        match &*term {
            Term::Var(x) => match self.env.lookup(&x) {
                Some(c) => {
                    self.env = c.env.clone();
                    self.term = c.term.clone();
                }
                None => return Err(format!("variable {} not in scope", x)),
            },
            Term::Lam(param, body) => match &*self.stack {
                Stack::Empty => return Ok(true),
                Stack::Cons(closure, stack) => {
                    self.term = body.clone();
                    self.env = self.env.extend(param.clone(), closure.clone());
                    self.stack = Rc::clone(stack);
                }
            },
            Term::App(fun, arg) => {
                self.term = fun.clone();
                self.stack = self.stack.extend(Closure::new(arg.clone(), self.env.clone()));
            },
        }
        Ok(false)
    }

    /// Run the machine to completion, returning the final Term or an error.
    /// Returns (term, env) that represents the final closure/value.
    pub fn evaluate(&mut self) -> Result<(Rc<Term>, Rc<Environment>), String> {
        loop {
            match self.step()? {
                true => return Ok((Rc::clone(&self.term), Rc::clone(&self.env))),
                false => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    // ---- helpers ----

    fn var(name: &str) -> Term {
        Term::Var(name.to_string())
    }

    fn rc_var(name: &str) -> Rc<Term> {
        Rc::new(var(name))
    }

    fn lam(param: &str, body: Term) -> Term {
        Term::Lam(param.to_string(), Rc::new(body))
    }

    fn app(fun: Term, arg: Term) -> Term {
        Term::App(Rc::new(fun), Rc::new(arg))
    }

    /// Evaluate a term to completion and return the resulting term
    fn eval(term: Term) -> Result<Term, String> {
        let mut machine = KrivMachine::new(term);
        let (term, _env) = machine.evaluate()?;
        Ok((*term).clone())
    }

    /// Evaluate and expect success, returning the term
    fn eval_ok(term: Term) -> Term {
        eval(term).expect("evaluation should succeed")
    }

    // ---- basic variable lookup ----

    #[test]
    fn test_var_not_in_scope_is_error() {
        let mut machine = KrivMachine::new(var("x"));
        let result = machine.step();
        assert!(result.is_err());
    }

    // ---- termination on lambda ----

    #[test]
    fn test_lam_with_empty_stack_terminates_immediately() {
        // A bare lambda is already a value
        let mut machine = KrivMachine::new(lam("x", var("x")));
        let result = machine.step().unwrap();
        assert!(result, "lam with empty stack should return true (done)");
    }

    #[test]
    fn test_nested_lam_terminates_immediately() {
        let mut machine = KrivMachine::new(lam("x", lam("y", var("x"))));
        let result = machine.step().unwrap();
        assert!(result, "nested lam with empty stack should return true (done)");
    }

    // ---- application steps ----

    #[test]
    fn test_app_pushes_closure_and_continues() {
        // (app f x) — should push x onto stack and continue with f
        let mut machine = KrivMachine::new(app(var("f"), var("x")));
        let result = machine.step().unwrap();
        assert!(!result, "app should not be done after one step");
    }

    #[test]
    fn test_identity_applied_to_identity() {
        // (λx.x) (λz.z) → λz.z
        let id = lam("z", var("z"));
        let term = app(lam("x", var("x")), id.clone());
        let result = eval_ok(term);
        // Result is the identity lambda
        match result {
            Term::Lam(param, body) => {
                assert_eq!(param, "z");
                match body.as_ref() {
                    Term::Var(name) => assert_eq!(name, "z"),
                    other => panic!("expected Var, got {:?}", other),
                }
            }
            other => panic!("expected Lam, got {:?}", other),
        }
    }

    #[test]
    fn test_constant_function() {
        // (λx.λy.x) (λa.a) (λb.b) → λa.a
        let id_a = lam("a", var("a"));
        let id_b = lam("b", var("b"));
        let term = app(app(lam("x", lam("y", var("x"))), id_a.clone()), id_b);
        let result = eval_ok(term);
        match result {
            Term::Lam(param, _body) => {
                assert_eq!(param, "a");
            }
            other => panic!("expected Lam(\"a\", ...), got {:?}", other),
        }
    }

    #[test]
    fn test_nested_application() {
        // (λx.x) ((λy.y) (λz.z)) → λz.z
        let id_z = lam("z", var("z"));
        let inner = app(lam("y", var("y")), id_z.clone());
        let term = app(lam("x", var("x")), inner);
        let result = eval_ok(term);
        match result {
            Term::Lam(param, body) => {
                assert_eq!(param, "z");
                match body.as_ref() {
                    Term::Var(name) => assert_eq!(name, "z"),
                    other => panic!("expected Var, got {:?}", other),
                }
            }
            other => panic!("expected Lam, got {:?}", other),
        }
    }

    // ---- omega combinator (self-application) ----

    #[test]
    fn test_self_application_single_step() {
        // (λx.x x) applied to itself — this loops forever, so just test it doesn't error immediately
        let omega_fun = lam("x", app(var("x"), var("x")));
        let omega_app = app(omega_fun.clone(), omega_fun);
        let mut machine = KrivMachine::new(omega_app);
        // Should be able to take many steps without error
        for _ in 0..100 {
            match machine.step() {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => panic!("unexpected error after some steps: {}", e),
            }
        }
    }

    // ---- closure capture ----

    #[test]
    fn test_closure_captures_environment() {
        // (λx.λy.x) z → λy.z  (closure captures z)
        let term = app(lam("x", lam("y", var("x"))), var("z"));
        let mut machine = KrivMachine::new(term);
        let (final_term, final_env) = machine.evaluate().unwrap();

        // Result is a lambda (the inner λy.x with x→z in the environment)
        match &*final_term {
            Term::Lam(param, _body) => {
                assert_eq!(param, "y");
            }
            other => panic!("expected Lam, got {:?}", other),
        }
        // The environment should have x bound
        let lookup = final_env.lookup("x");
        assert!(lookup.is_some());
    }

    // ---- free variable at runtime ----

    #[test]
    fn test_free_variable_in_body_is_error_at_runtime() {
        // (λx.z) y — z is free in body, so when body is evaluated z isn't found
        // Step 1: App pushes y as closure and moves to lam
        // Step 2: Lam sees stack has y, extends env with x→y, body=z
        // Step 3: z is looked up but not in scope → error
        let term = app(lam("x", var("z")), var("y"));
        let result = eval(term);
        assert!(result.is_err());
    }

    #[test]
    fn test_free_variable_in_function_position_is_error() {
        // (z x) — z is free
        let term = app(var("z"), var("x"));
        let result = eval(term);
        assert!(result.is_err());
    }

    // ---- shadowing ----

    #[test]
    fn test_variable_shadowing() {
        // (λx.(λx.x) (λy.y)) (λz.z)
        // Inner identity (λx.x) applied to (λy.y) → λy.y
        // Outer ignores its arg (λz.z) since x is shadowed; result is λy.y
        let id_y = lam("y", var("y"));
        let id_z = lam("z", var("z"));
        let term = app(
            lam("x", app(lam("x", var("x")), id_y.clone())),
            id_z,
        );
        let result = eval_ok(term);
        match result {
            Term::Lam(param, body) => {
                assert_eq!(param, "y");
                match body.as_ref() {
                    Term::Var(name) => assert_eq!(name, "y"),
                    other => panic!("expected Var, got {:?}", other),
                }
            }
            other => panic!("expected Lam, got {:?}", other),
        }
    }

    // ---- multi-argument simulation (currying) ----

    #[test]
    fn test_curried_three_args() {
        // λx.λy.λz.z  applied to id id id → id (λz.z)
        let id = lam("z", var("z"));
        let term = app(
            app(
                app(lam("x", lam("y", lam("z", var("z")))), id.clone()),
                id.clone(),
            ),
            id.clone(),
        );
        let result = eval_ok(term);
        // Result is λz.z
        match result {
            Term::Lam(param, body) => {
                assert_eq!(param, "z");
                match body.as_ref() {
                    Term::Var(name) => assert_eq!(name, "z"),
                    other => panic!("expected Var, got {:?}", other),
                }
            }
            other => panic!("expected Lam, got {:?}", other),
        }
    }

    // ---- step counting ----

    #[test]
    fn test_identity_application_step_count() {
        // (λx.x) (λz.z)  takes: App step (1) + Lam step (2) → done at step 2
        let id = lam("z", var("z"));
        let mut machine = KrivMachine::new(app(lam("x", var("x")), id));
        let mut steps = 0;
        loop {
            steps += 1;
            match machine.step().unwrap() {
                true => break,
                false => continue,
            }
        }
        assert_eq!(steps, 4, "identity application should finish in 4 steps");
    }

    #[test]
    fn test_double_application_step_count() {
        // (λx.λy.x) (λa.a) (λb.b)
        let id_a = lam("a", var("a"));
        let id_b = lam("b", var("b"));
        let mut machine = KrivMachine::new(app(
            app(lam("x", lam("y", var("x"))), id_a),
            id_b,
        ));
        let mut steps = 0;
        loop {
            steps += 1;
            match machine.step().unwrap() {
                true => break,
                false => continue,
            }
        }
        assert_eq!(steps, 6, "double application should finish in 6 steps");
    }

    // ---- machine::new starts with correct state ----

    #[test]
    fn test_new_machine_has_empty_env_and_stack() {
        let term = lam("x", var("x"));
        let machine = KrivMachine::new(term);
        match &*machine.env {
            Environment::Empty => {}
            _ => panic!("expected Empty environment"),
        }
        match &*machine.stack {
            Stack::Empty => {}
            _ => panic!("expected Empty stack"),
        }
    }

    // ---- machine state transitions ----

    #[test]
    fn test_app_step_changes_term_to_fun() {
        let mut machine = KrivMachine::new(app(var("f"), var("x")));
        machine.step().unwrap();
        // Term should now be f
        match &*machine.term {
            Term::Var(name) => assert_eq!(name, "f"),
            other => panic!("expected Var(\"f\"), got {:?}", other),
        }
    }

    #[test]
    fn test_app_step_pushes_arg_onto_stack() {
        let mut machine = KrivMachine::new(app(var("f"), var("x")));
        machine.step().unwrap();
        // Stack should have one element (the arg closure)
        match &*machine.stack {
            Stack::Cons(closure, tail) => {
                match &*closure.term {
                    Term::Var(name) => assert_eq!(name, "x"),
                    other => panic!("expected Var(\"x\"), got {:?}", other),
                }
                match &**tail {
                    Stack::Empty => {}
                    _ => panic!("expected Empty tail"),
                }
            }
            _ => panic!("expected Cons on stack"),
        }
    }

    #[test]
    fn test_lam_with_closure_on_stack_applies_it() {
        // Create a machine where term=Lam(x, x) and stack has closure(x→a)
        let a_closure = Closure::new(rc_var("a"), Rc::new(Environment::Empty));
        let stack = Rc::new(Stack::Empty).extend(a_closure);
        let mut machine = KrivMachine {
            term: Rc::new(lam("x", var("x"))),
            env: Rc::new(Environment::Empty),
            stack,
        };
        let done = machine.step().unwrap();
        assert!(!done, "should continue after applying lambda");
        // Term should now be the body: x
        match &*machine.term {
            Term::Var(name) => assert_eq!(name, "x"),
            other => panic!("expected Var(\"x\"), got {:?}", other),
        }
        // Env should have x bound
        assert!(machine.env.lookup("x").is_some());
    }

    #[test]
    fn test_var_lookup_substitutes_env() {
        // Manually set up: term=Var("x"), env has x→Identity
        let identity_closure = Closure::new(
            Rc::new(lam("y", var("y"))),
            Rc::new(Environment::Empty),
        );
        let env = Rc::new(Environment::Empty).extend("x".to_string(), identity_closure);
        let mut machine = KrivMachine {
            term: rc_var("x"),
            env,
            stack: Rc::new(Stack::Empty),
        };
        let done = machine.step().unwrap();
        assert!(!done, "should continue after var lookup (need to evaluate closure's term)");
        // Term should now be the identity lambda
        match &*machine.term {
            Term::Lam(param, _) => assert_eq!(param, "y"),
            other => panic!("expected Lam, got {:?}", other),
        }
    }
}
