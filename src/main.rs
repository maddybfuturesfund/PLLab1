use im::HashMap;
use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

/// Unary operators
#[derive(Debug)]
enum Op1 {
    Add1,
    Sub1,
}

/// Binary operators
#[derive(Debug)]
enum Op2 {
    Plus,
    Minus,
    Times,
}

/// The Boa expression AST
///
/// Grammar:
///   <expr> := <number>
///           | <identifier>
///           | (let (<binding>+) <expr>)
///           | (add1 <expr>) | (sub1 <expr>)
///           | (+ <expr> <expr>) | (- <expr> <expr>) | (* <expr> <expr>)
///   <binding> := (<identifier> <expr>)
#[derive(Debug)]
enum Expr {
    Number(i32),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
}

// ============= Assembly Representation =============

/// Values that can appear in assembly instructions
#[derive(Debug)]
enum Val {
    Reg(Reg),
    Imm(i32),
    RegOffset(Reg, i32), // e.g., [rsp - 8]
}

/// Registers we use
#[derive(Debug)]
enum Reg {
    RAX,
    RSP,
}

/// Assembly instructions we generate
#[derive(Debug)]
enum Instr {
    IMov(Val, Val),
    IAdd(Val, Val),
    ISub(Val, Val),
    IMul(Val, Val),
}


// ============= Parsing =============

/// Parse an S-expression into our Expr AST
///
/// Examples of valid Boa expressions:
///   42                          -> Number(42)
///   x                           -> Id("x")
///   (add1 5)                    -> UnOp(Add1, Number(5))
///   (+ 1 2)                     -> BinOp(Plus, Number(1), Number(2))
///   (let ((x 5)) x)             -> Let([("x", Number(5))], Id("x"))
///   (let ((x 5) (y 6)) (+ x y)) -> Let([("x", Number(5)), ("y", Number(6))], BinOp(...))
///
/// Error handling:
///   - Invalid syntax: panic!("Invalid")
///   - Number out of i32 range: panic!("Invalid")
fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(F(_)) => panic!("Invalid: Floats not supported"),
        // TODO: Handle number atoms
        // Hint: Sexp::Atom(I(n)) => ...
        //       Use i32::try_from(*n).unwrap_or_else(|_| panic!("Invalid"))
        Sexp::Atom(I(n)) =>  {
            Expr::Number(i32::try_from(*n).unwrap_or_else(|_| panic!("Invalid")))
        }

        // TODO: Handle identifier atoms
        // Hint: Sexp::Atom(S(name)) => ...
        //       Make sure to check it's not a reserved keyword
        Sexp::Atom(S(name)) => {
            match name.as_str() {
                "add1" | "sub1" | "let" => panic!("Invalid: reserved keyword used as identifier"),
                _ => Expr::Id(name.clone()),
            }
        }

        // TODO: Handle list expressions
        // Hint: Sexp::List(vec) => match &vec[..] { ... }
        //
        // Cases to handle:
        //   [Sexp::Atom(S(op)), e] if op == "add1" => UnOp(Add1, ...)
        //   [Sexp::Atom(S(op)), e] if op == "sub1" => UnOp(Sub1, ...)
        //   [Sexp::Atom(S(op)), e1, e2] if op == "+" => BinOp(Plus, ...)
        //   [Sexp::Atom(S(op)), e1, e2] if op == "-" => BinOp(Minus, ...)
        //   [Sexp::Atom(S(op)), e1, e2] if op == "*" => BinOp(Times, ...)
        //   [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => ...
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(op)), e] if op == "add1" => Expr::UnOp(Op1::Add1, Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e] if op == "sub1" => Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e1, e2] if op == "+" => Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "-" => Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "*" => Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => {
                let parsed_bindings = bindings.iter().map(|b| parse_bind(b)).collect();
                Expr::Let(parsed_bindings, Box::new(parse_expr(body)))
            }

        _ => panic!("Invalid: unrecognized expression"),
        }
    }

}

/// Parse a single binding from a let expression
///
/// A binding looks like: (x 5) or (my-var (+ 1 2))
/// Returns a tuple of (variable_name, expression)
///
/// Error handling:
///   - Invalid binding syntax: panic!("Invalid")
fn parse_bind(s: &Sexp) -> (String, Expr) {
    // TODO: Parse a binding of the form (identifier expr)
    // Hint: match s {
    //     Sexp::List(vec) => match &vec[..] {
    //         [Sexp::Atom(S(name)), e] => (name.clone(), parse_expr(e)),
    //         _ => panic!("Invalid"),
    //     }
    //     _ => panic!("Invalid"),
    // }
        match s {
            Sexp::List(vec) => match &vec[..] {
                [Sexp::Atom(S(name)), e] => (name.clone(), parse_expr(e)),
                _ => panic!("Invalid: binding must be of the form (identifier expr)"),
            },
            _ => panic!("Invalid: binding must be a list"),
        }


    //panic!("TODO: Implement parse_bind")
}

fn compile_to_instrs(e: &Expr, si: i32, env: &HashMap<String, i32>) -> Vec<Instr> {
    match e {
        // TODO: Number - move immediate value to RAX
        // vec![IMov(Val::Reg(Reg::RAX), Val::Imm(*n))]
        Expr::Number(n) => { 
            vec![Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(*n))]
        }   

        // TODO: Id - look up variable in environment, load from stack
        // 1. Look up name in env to get stack offset
        //    env.get(name).unwrap_or_else(|| panic!("Unbound variable identifier {}", name))
        // 2. Generate: IMov(Reg(RAX), RegOffset(RSP, offset))
        Expr::Id(name) => {
            let offset = env.get(name).unwrap_or_else(|| panic!("Unbound variable identifier {}", name));
            vec![Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, *offset))]
        }

        // TODO: UnOp - compile subexpression, then apply operation
        // Add1: compile e, then IAdd(Reg(RAX), Imm(1))
        // Sub1: compile e, then ISub(Reg(RAX), Imm(1))
        Expr::UnOp(op, e) => compile_to_instrs(e, si, env).into_iter().chain(
            match op {
                Op1::Add1 => vec![Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(1))],
                Op1::Sub1 => vec![Instr::ISub(Val::Reg(Reg::RAX), Val::Imm(1))],
            }
        ).collect(),

        // TODO: BinOp - compile both operands using the stack
        // Strategy:
        //   1. Compile left operand (result in RAX)
        //   2. Store RAX at [rsp - 8*si] (save left result)
        //   3. Compile right operand with si+1 (result in RAX)
        //   4. Perform operation: stack_value OP rax -> rax
        //
        // For Plus:  load left from stack, add rax
        // For Minus: load left from stack, sub rax
        // For Times: load left from stack, mul rax
        //
        // Hint: You may need to move the left operand back to RAX
        //       and then apply the operation with the right operand
        Expr::BinOp(op, left, right) => {
            let mut instrs = compile_to_instrs(left, si, env);
            instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, -8*si), Val::Reg(Reg::RAX)));
            instrs.extend(compile_to_instrs(right, si+1, env));
            let offset = -8*si;
            match op {
                Op2::Plus => instrs.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset))),
                Op2::Minus => {
                    instrs.push(Instr::ISub(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset)));
                },
                Op2::Times => instrs.push(Instr::IMul(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset))),
            }
            instrs
        }

        // TODO: Let - bind variables and compile body
        // Strategy:
        //   1. Check for duplicate bindings - panic!("Duplicate binding")
        //   2. For each binding (name, expr):
        //      a. Compile expr with current si and env
        //      b. Store result at [rsp - 8*si]
        //      c. Add name -> -8*si to env
        //      d. Increment si
        //   3. Compile body with final si and env
        //
        // Duplicate check: Keep a set of names seen so far
        // If you see a name twice, panic!("Duplicate binding")
        Expr::Let(bindings, body) => {
            let mut instrs = Vec::new();
            let mut seen = HashMap::new();
            let mut current_env = env.clone();
            let mut current_si = si;

            for (name, expr) in bindings {
                if seen.contains_key(name) {
                    panic!("Duplicate binding");
                }
                seen.insert(name.clone(), true);

                instrs.extend(compile_to_instrs(expr, current_si, &current_env));
                instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, -8*current_si), Val::Reg(Reg::RAX)));
                current_env.insert(name.clone(), -8*current_si);
                current_si += 1;
            }

            instrs.extend(compile_to_instrs(body, current_si, &current_env));
            instrs
        }

        _ => panic!("TODO: Implement compile_to_instrs for {:?}", e),
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.snek> <output.s>", args[0]);
        std::process::exit(1);
    }

    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    // Parse S-expression from text
    let sexp = parse(&in_contents).unwrap_or_else(|_| panic!("Invalid"));

    // Convert S-expression to our AST
    let expr = parse_expr(&sexp);

    // Generate assembly instructions
    let instrs = compile(&expr);

    // Wrap instructions in assembly program template
    let asm_program = format!(
        "section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
",
        instrs
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}



// ============= Code Generation =============

/// Convert a Val to its assembly string representation
fn val_to_str(v: &Val) -> String {
    match v {
        Val::Reg(Reg::RAX) => String::from("rax"),
        Val::Reg(Reg::RSP) => String::from("rsp"),
        Val::Imm(n) => format!("{}", n),
        Val::RegOffset(Reg::RSP, offset) => format!("[rsp + {}]", offset),
        Val::RegOffset(Reg::RAX, offset) => format!("[rax + {}]", offset),
    }
}

/// Convert an Instr to its assembly string representation
fn instr_to_str(i: &Instr) -> String {
    match i {
        Instr::IMov(dst, src) => format!("mov {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::IAdd(dst, src) => format!("add {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::ISub(dst, src) => format!("sub {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::IMul(dst, src) => format!("imul {}, {}", val_to_str(dst), val_to_str(src)),
    }
}

/// Compile an expression to a complete assembly string
fn compile(e: &Expr) -> String {
    let env: HashMap<String, i32> = HashMap::new();
    let instrs = compile_to_instrs(e, 2, &env);
    instrs
        .iter()
        .map(|i| instr_to_str(i))
        .collect::<Vec<String>>()
        .join("\n  ")
}


#[cfg(test)]
mod tests {
    use super::*;

    // Helper to parse a string directly
    fn parse_str(s: &str) -> Expr {
        parse_expr(&parse(s).unwrap())
    }

    // ===== Parsing Tests =====

    #[test]
    fn test_parse_number() {
        let expr = parse_str("42");
        match expr {
            Expr::Number(42) => (),
            _ => panic!("Expected Number(42), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_identifier() {
        let expr = parse_str("x");
        match expr {
            Expr::Id(name) => assert_eq!(name, "x"),
            _ => panic!("Expected Id(\"x\"), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_add1() {
        let expr = parse_str("(add1 5)");
        match expr {
            Expr::UnOp(Op1::Add1, _) => (),
            _ => panic!("Expected UnOp(Add1, ...), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_binary_plus() {
        let expr = parse_str("(+ 1 2)");
        match expr {
            Expr::BinOp(Op2::Plus, _, _) => (),
            _ => panic!("Expected BinOp(Plus, ...), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_let_simple() {
        let expr = parse_str("(let ((x 5)) x)");
        match expr {
            Expr::Let(bindings, _) => {
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].0, "x");
            }
            _ => panic!("Expected Let, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_let_multiple_bindings() {
        let expr = parse_str("(let ((x 5) (y 6)) (+ x y))");
        match expr {
            Expr::Let(bindings, _) => {
                assert_eq!(bindings.len(), 2);
            }
            _ => panic!("Expected Let with 2 bindings, got {:?}", expr),
        }
    }

    // ===== Error Tests =====

    #[test]
    #[should_panic(expected = "Duplicate binding")]
    fn test_duplicate_binding() {
        let expr = parse_str("(let ((x 1) (x 2)) x)");
        let env: HashMap<String, i32> = HashMap::new();
        compile_to_instrs(&expr, 2, &env);
    }

    #[test]
    #[should_panic(expected = "Unbound variable identifier y")]
    fn test_unbound_variable() {
        let expr = parse_str("y");
        let env: HashMap<String, i32> = HashMap::new();
        compile_to_instrs(&expr, 2, &env);
    }

    // ===== Compilation Tests =====

    #[test]
    fn test_compile_number() {
        let expr = Expr::Number(42);
        let env: HashMap<String, i32> = HashMap::new();
        let instrs = compile_to_instrs(&expr, 2, &env);
        assert_eq!(instrs.len(), 1);
    }

    // Add more tests as you implement features!
}
