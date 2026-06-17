// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// wasm-lambda
// src/machine.rs
//
// Makabaka1880, 2026. All rights reserved.

use std::rc::Rc;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum Term {
    Var(String),
    Lam(String, Rc<Term>),
    App(Rc<Term>, Rc<Term>),
}

#[derive(Clone, Debug, Serialize)]
pub struct Closure {
    pub term: Rc<Term>,
    pub env: Rc<Environment>,
}

impl Closure {
    pub fn new(term: Rc<Term>, env: Rc<Environment>) -> Self {
        Closure { term, env }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum Stack {
    Empty,
    Cons(Closure, Rc<Stack>),
}

#[derive(Clone, Debug, Serialize)]
pub enum Environment {
    Empty,
    Cons(String, Closure, Rc<Environment>),
}

impl Stack {
    pub fn extend(self: &Rc<Self>, closure: Closure) -> Rc<Self> {
        Rc::new(Stack::Cons(closure, Rc::clone(self)))
    }
}

impl Environment {
    pub fn extend(self: &Rc<Self>, name: String, closure: Closure) -> Rc<Self> {
        Rc::new(Environment::Cons(name, closure, Rc::clone(self)))
    }

    pub fn lookup(self: &Rc<Self>, name: &str) -> Option<Closure> {
        match &**self {
            Environment::Empty => None,
            Environment::Cons(key, closure, parent) => {
                if key == name {
                    Some(closure.clone())
                } else {
                    parent.lookup(name)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn var(name: &str) -> Rc<Term> {
        Rc::new(Term::Var(name.to_string()))
    }

    fn dummy_closure() -> Closure {
        Closure {
            term: var("x"),
            env: Rc::new(Environment::Empty),
        }
    }

    #[test]
    fn test_empty_env_lookup_returns_none() {
        let env = Rc::new(Environment::Empty);
        assert!(env.lookup("x").is_none());
        assert!(env.lookup("y").is_none());
    }

    #[test]
    fn test_env_lookup_finds_binding() {
        let closure = dummy_closure();
        let env = Rc::new(Environment::Empty);
        let env = env.extend("x".to_string(), closure.clone());
        let found = env.lookup("x");
        assert!(found.is_some());
    }

    #[test]
    fn test_env_lookup_returns_none_for_missing() {
        let closure = dummy_closure();
        let env = Rc::new(Environment::Empty);
        let env = env.extend("x".to_string(), closure);
        assert!(env.lookup("y").is_none());
    }

    #[test]
    fn test_env_lookup_respects_shadowing() {
        let inner_closure = Closure {
            term: var("inner"),
            env: Rc::new(Environment::Empty),
        };
        let outer_closure = Closure {
            term: var("outer"),
            env: Rc::new(Environment::Empty),
        };
        let env = Rc::new(Environment::Empty);
        let env = env.extend("x".to_string(), outer_closure);
        let env = env.extend("x".to_string(), inner_closure.clone());
        let found = env.lookup("x");
        assert!(found.is_some());
        // Should find the inner (most recent) binding
        match &*found.unwrap().term {
            Term::Var(name) => assert_eq!(name, "inner"),
            _ => panic!("expected Var"),
        }
    }

    #[test]
    fn test_env_lookup_chain() {
        let c1 = Closure {
            term: var("a"),
            env: Rc::new(Environment::Empty),
        };
        let c2 = Closure {
            term: var("b"),
            env: Rc::new(Environment::Empty),
        };
        let c3 = Closure {
            term: var("c"),
            env: Rc::new(Environment::Empty),
        };
        let env = Rc::new(Environment::Empty);
        let env = env.extend("a".to_string(), c1);
        let env = env.extend("b".to_string(), c2);
        let env = env.extend("c".to_string(), c3);

        assert!(env.lookup("a").is_some());
        assert!(env.lookup("b").is_some());
        assert!(env.lookup("c").is_some());
        assert!(env.lookup("d").is_none());
    }

    #[test]
    fn test_stack_extend() {
        let stack = Rc::new(Stack::Empty);
        let closure = dummy_closure();
        let extended = stack.extend(closure);
        match &*extended {
            Stack::Cons(_, tail) => match &**tail {
                Stack::Empty => {} // correct
                _ => panic!("expected Empty tail"),
            },
            _ => panic!("expected Cons"),
        }
    }

    #[test]
    fn test_closure_new() {
        let term = var("t");
        let env = Rc::new(Environment::Empty);
        let closure = Closure::new(Rc::clone(&term), Rc::clone(&env));
        assert!(Rc::ptr_eq(&closure.term, &term));
        assert!(Rc::ptr_eq(&closure.env, &env));
    }

    #[test]
    fn test_term_clone() {
        let term = Term::Var("x".to_string());
        let cloned = term.clone();
        match (term, cloned) {
            (Term::Var(a), Term::Var(b)) => assert_eq!(a, b),
            _ => panic!("expected Vars"),
        }
    }

    #[test]
    fn test_term_debug_format() {
        let var = Term::Var("x".to_string());
        let lam = Term::Lam("x".to_string(), Rc::new(Term::Var("x".to_string())));
        let app = Term::App(
            Rc::new(Term::Var("f".to_string())),
            Rc::new(Term::Var("x".to_string())),
        );
        // Verify Debug is implemented and doesn't panic
        let _ = format!("{:?}", var);
        let _ = format!("{:?}", lam);
        let _ = format!("{:?}", app);
    }

    #[test]
    fn test_environment_clone() {
        let env = Rc::new(Environment::Empty);
        assert_eq!(Rc::strong_count(&env), 1);
        let cloned = Rc::clone(&env);
        assert_eq!(Rc::strong_count(&env), 2);
        drop(cloned);
        assert_eq!(Rc::strong_count(&env), 1);
    }
}
