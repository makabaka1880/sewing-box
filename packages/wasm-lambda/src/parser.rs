// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// wasm-lambda
// src/parser.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::machine::Term;
use sexp::*;

pub fn parse(src: Sexp) -> Result<Term, String> {
    let list = match src {
        Sexp::Atom(atom) => return Ok(Term::Var(atom.to_string())),
        Sexp::List(sexps) => sexps,
    };
    match list.as_slice() {
        [op, arg, body] => match op {
            Sexp::Atom(Atom::S(s)) if s == "lam" => {
                let param = match arg {
                    Sexp::Atom(a) => a.to_string(),
                    _ => return Err("expected atom as lambda parameter".into()),
                };
                let body_term = parse(body.clone())?;
                Ok(Term::Lam(param, body_term.into()))
            }
            Sexp::Atom(Atom::S(s)) if s == "app" => {
                let fun = parse(arg.clone())?;
                let arg_term = parse(body.clone())?;
                Ok(Term::App(fun.into(), arg_term.into()))
            }
            _ => Err(format!("unknown operator: {:?}", op)),
        },
        _ => Err(format!("expected list of 3 elements, got {:?}", list)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse an s-expression string into a Term
    fn parse_str(s: &str) -> Result<Term, String> {
        let sexp = sexp::parse(s).map_err(|e| format!("sexp parse error: {:?}", e))?;
        parse(sexp)
    }

    /// Helper: verify a term is Var with the given name
    fn assert_var(term: &Term, expected: &str) {
        match term {
            Term::Var(name) => assert_eq!(name, expected),
            other => panic!("expected Var, got {:?}", other),
        }
    }

    /// Helper: verify a term is Lam and call f with param and body
    fn assert_lam<F>(term: &Term, f: F)
    where
        F: FnOnce(&str, &Term),
    {
        match term {
            Term::Lam(param, body) => f(param, body.as_ref()),
            other => panic!("expected Lam, got {:?}", other),
        }
    }

    /// Helper: verify a term is App and call f with fun and arg
    fn assert_app<F>(term: &Term, f: F)
    where
        F: FnOnce(&Term, &Term),
    {
        match term {
            Term::App(fun, arg) => f(fun.as_ref(), arg.as_ref()),
            other => panic!("expected App, got {:?}", other),
        }
    }

    // ---- variable (atom) tests ----

    #[test]
    fn test_parse_variable() {
        let term = parse_str("x").unwrap();
        assert_var(&term, "x");
    }

    #[test]
    fn test_parse_variable_long_name() {
        let term = parse_str("foobar").unwrap();
        assert_var(&term, "foobar");
    }

    // ---- lambda tests ----

    #[test]
    fn test_parse_simple_lambda() {
        let term = parse_str("(lam x x)").unwrap();
        assert_lam(&term, |param, body| {
            assert_eq!(param, "x");
            assert_var(body, "x");
        });
    }

    #[test]
    fn test_parse_lambda_var_body_is_itself() {
        // Identity function: λx.x
        let term = parse_str("(lam x x)").unwrap();
        assert_lam(&term, |param, body| {
            assert_eq!(param, "x");
            assert_var(body, "x");
        });
    }

    #[test]
    fn test_parse_lambda_with_different_param_and_body() {
        // λx.y  (constant function, body is a free variable)
        let term = parse_str("(lam x y)").unwrap();
        assert_lam(&term, |param, body| {
            assert_eq!(param, "x");
            assert_var(body, "y");
        });
    }

    #[test]
    fn test_parse_nested_lambda() {
        // λx.λy.x
        let term = parse_str("(lam x (lam y x))").unwrap();
        assert_lam(&term, |x_param, inner| {
            assert_eq!(x_param, "x");
            assert_lam(inner, |y_param, body| {
                assert_eq!(y_param, "y");
                assert_var(body, "x");
            });
        });
    }

    #[test]
    fn test_parse_double_nested_lambda() {
        // λx.λy.λz.z
        let term = parse_str("(lam x (lam y (lam z z)))").unwrap();
        assert_lam(&term, |x, mid| {
            assert_eq!(x, "x");
            assert_lam(mid, |y, inner| {
                assert_eq!(y, "y");
                assert_lam(inner, |z, body| {
                    assert_eq!(z, "z");
                    assert_var(body, "z");
                });
            });
        });
    }

    // ---- application tests ----

    #[test]
    fn test_parse_simple_application() {
        // (f x)
        let term = parse_str("(app f x)").unwrap();
        assert_app(&term, |fun, arg| {
            assert_var(fun, "f");
            assert_var(arg, "x");
        });
    }

    #[test]
    fn test_parse_nested_application() {
        // ((f x) y)
        let term = parse_str("(app (app f x) y)").unwrap();
        assert_app(&term, |fun, arg| {
            assert_app(fun, |inner_fun, inner_arg| {
                assert_var(inner_fun, "f");
                assert_var(inner_arg, "x");
            });
            assert_var(arg, "y");
        });
    }

    #[test]
    fn test_parse_curried_application() {
        // f x y  as  (app (app f x) y)
        let term = parse_str("(app (app f x) y)").unwrap();
        assert!(matches!(term, Term::App(_, _)));
    }

    // ---- mixed lambda + application tests ----

    #[test]
    fn test_parse_lambda_application() {
        // (λx.x) y
        let term = parse_str("(app (lam x x) y)").unwrap();
        assert_app(&term, |fun, arg| {
            assert_lam(fun, |param, body| {
                assert_eq!(param, "x");
                assert_var(body, "x");
            });
            assert_var(arg, "y");
        });
    }

    #[test]
    fn test_parse_application_inside_lambda() {
        // λx.(f x)
        let term = parse_str("(lam x (app f x))").unwrap();
        assert_lam(&term, |param, body| {
            assert_eq!(param, "x");
            assert_app(body, |fun, arg| {
                assert_var(fun, "f");
                assert_var(arg, "x");
            });
        });
    }

    // ---- error cases ----

    #[test]
    fn test_parse_empty_list_is_error() {
        let result = parse_str("()");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_two_element_list_is_error() {
        let result = parse_str("(lam x)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_four_element_list_is_error() {
        let result = parse_str("(lam x y z)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_operator_is_error() {
        let result = parse_str("(foo x y)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_lam_with_non_atom_param_is_error() {
        // (lam (x) x) — parameter must be an atom, not a list
        let result = parse_str("(lam (x) x)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_lam_with_app_as_param_is_error() {
        // (lam (app a b) x)
        let result = parse_str("(lam (app a b) x)");
        assert!(result.is_err());
    }
}
