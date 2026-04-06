use im::HashMap;
use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

const NUM_TAG: i64 = 0;
const NUM_TAG_MASK: i64 = 1;
const BOOL_TAG: i64 = 1;
const BOOL_TAG_MASK: i64 = 1;
const TRUE_VAL: i64 = 3;
const FALSE_VAL: i64 = 1;

fn encode_num(n: i32) -> i64 {
    (n as i64) << 1
}

fn decode_num(tagged: i64) -> i32 {
    (tagged >> 1) as i32
}

/// Unary operators
#[derive(Debug)]
enum UnOp {
    Add1, Sub1, Negate,
    IsNum, IsBool,
}

/// Binary operators
#[derive(Debug)]
enum BinOp {
    Plus, Minus, Times,
    Less, Greater, LessEq, GreaterEq, Equal,
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
    Bool(bool),
    Id(String),
    Input,
    Var(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    Loop(Box<Expr>),
    Break(Box<Expr>),
    Set(String, Box<Expr>),
}

// ============= Assembly Representation =============

/// Values that can appear in assembly instructions
#[derive(Debug, Clone, Copy)]
enum Val {
    Reg(Reg),
    Imm(i64),
    RegOffset(Reg, i32), // e.g., [rsp - 8]
}

/// Registers we use
#[derive(Debug, Clone, Copy)]
enum Reg {
    RAX,
    RSP,
    RBX,
    RDI,
}

/// Assembly instructions we generate
#[derive(Debug)]
enum Instr {
    IMov(Val, Val),
    IAdd(Val, Val),
    ISub(Val, Val),
    IMul(Val, Val),
    ICmp(Val, Val),
    IJmp(String),
    IJne(String),
    ILabel(String),
    IAnd(Val, Val),
    IJge(String),
    IJle(String),
    IJg(String),
    IJl(String),
    ISar(Val, Val),
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

        Sexp::Atom(S(name)) if name == "true" => Expr::Bool(true),
        Sexp::Atom(S(name)) if name == "false" => Expr::Bool(false),

        // TODO: Handle identifier atoms
        // Hint: Sexp::Atom(S(name)) => ...
        //       Make sure to check it's not a reserved keyword
        Sexp::Atom(S(name)) => {
            match name.as_str() {
                "add1" | "sub1" | "let" | "loop" | "break" | "if" | "block" => panic!("Invalid: reserved keyword used as identifier"),
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
            [Sexp::Atom(S(op)), e] if op == "add1" => Expr::UnOp(UnOp::Add1, Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e] if op == "sub1" => Expr::UnOp(UnOp::Sub1, Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e1, e2] if op == "+" => Expr::BinOp(BinOp::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "-" => Expr::BinOp(BinOp::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "*" => Expr::BinOp(BinOp::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "<" => Expr::BinOp(BinOp::Less, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == ">" => Expr::BinOp(BinOp::Greater, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "<=" => Expr::BinOp(BinOp::LessEq, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == ">=" => Expr::BinOp(BinOp::GreaterEq, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "=" => Expr::BinOp(BinOp::Equal, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), cond, thn, els] if op == "if" => {
                Expr::If(Box::new(parse_expr(cond)), Box::new(parse_expr(thn)), Box::new(parse_expr(els)))
            },
        
            // Handle (set! <name> <val>)
            [Sexp::Atom(S(op)), Sexp::Atom(S(name)), e] if op == "set!" => {
                Expr::Set(name.clone(), Box::new(parse_expr(e)))
            },
        
            // Handle (loop <body>)
            [Sexp::Atom(S(op)), body] if op == "loop" => {
                Expr::Loop(Box::new(parse_expr(body)))
            },
        
            // Handle (break <val>)
            [Sexp::Atom(S(op)), val] if op == "break" => {
                Expr::Break(Box::new(parse_expr(val)))
            },

            [Sexp::Atom(S(op)), exprs @ ..] if op == "block" => {
                Expr::Block(exprs.iter().map(parse_expr).collect())
            },
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

fn compile_to_instrs(e: &Expr, si: i32, env: &HashMap<String, i32>, label_counter: &mut i32, break_target: &Option<String>) -> Vec<Instr> {
    match e {
        // TODO: Number - move immediate value to RAX
        // vec![IMov(Val::Reg(Reg::RAX), Val::Imm(*n))]
        Expr::Number(n) => { 
            vec![Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(encode_num(*n)))]
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
        Expr::UnOp(op, e) => compile_to_instrs(e, si, env, label_counter, break_target).into_iter().chain(
            match op {
                UnOp::Add1 => vec![Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(encode_num(1)))],
                UnOp::Sub1 => vec![Instr::ISub(Val::Reg(Reg::RAX), Val::Imm(encode_num(1)))],   
                UnOp::Negate => vec![Instr::IMul(Val::Reg(Reg::RAX), Val::Imm(-1))],
                UnOp::IsNum => {
                    let skip_label = new_label(label_counter, "isnum_skip");
                    vec![
                        Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)),
                        Instr::IAnd(Val::Reg(Reg::RBX), Val::Imm(NUM_TAG_MASK)),
                        Instr::ICmp(Val::Reg(Reg::RBX), Val::Imm(NUM_TAG)),
                        Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)), // Default to False
                        Instr::IJne(skip_label.clone()),                     // If not a num, jump to end
                        Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)),  // Otherwise, set to True
                        Instr::ILabel(skip_label),
                    ]
                },
                UnOp::IsBool => {
                    let skip_label = new_label(label_counter, "isbool_skip");
                    vec![
                        Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)),
                        Instr::IAnd(Val::Reg(Reg::RBX), Val::Imm(BOOL_TAG_MASK)),
                        Instr::ICmp(Val::Reg(Reg::RBX), Val::Imm(BOOL_TAG)),
                        Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)),
                        Instr::IJne(skip_label.clone()),
                        Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)),
                        Instr::ILabel(skip_label),
                    ]
                },
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
            let mut instrs = compile_to_instrs(left, si, env, label_counter, break_target);
            let offset = -8*si;

            instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)));
            instrs.push(Instr::IAnd(Val::Reg(Reg::RBX), Val::Imm(1)));
            instrs.push(Instr::ICmp(Val::Reg(Reg::RBX), Val::Imm(0)));
            instrs.push(Instr::IJne("throw_error".to_string()));

            instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, offset), Val::Reg(Reg::RAX)));
            instrs.extend(compile_to_instrs(right, si+1, env, label_counter, break_target));

            instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX)));
            instrs.push(Instr::IAnd(Val::Reg(Reg::RBX), Val::Imm(1)));
            instrs.push(Instr::ICmp(Val::Reg(Reg::RBX), Val::Imm(0)));
            instrs.push(Instr::IJne("throw_error".to_string()));
            match op {
                BinOp::Plus => instrs.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset))),
                BinOp::Minus => {
                    instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::Reg(Reg::RAX))); 
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset))); 
                    instrs.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Reg(Reg::RBX)));
                },
                BinOp::Times => {
                    instrs.push(Instr::IMov(Val::Reg(Reg::RBX), Val::RegOffset(Reg::RSP, offset)));
                    instrs.push(Instr::IMul(Val::Reg(Reg::RAX), Val::Reg(Reg::RBX)));
                    instrs.push(Instr::ISar(Val::Reg(Reg::RAX), Val::Imm(1)));
                },
                BinOp::Less => {
                    let else_label = new_label(label_counter, "less_else");
                    let end_label = new_label(label_counter, "less_end");

                    instrs.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset)));
                    instrs.push(Instr::IJge(else_label.clone()));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
                    instrs.push(Instr::IJmp(end_label.clone()));
                    instrs.push(Instr::ILabel(else_label));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)));
                    instrs.push(Instr::ILabel(end_label));
                },
                BinOp::Greater => {
                    let else_label = new_label(label_counter, "greater_else");
                    let end_label = new_label(label_counter, "greater_end");

                    instrs.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset)));
                    instrs.push(Instr::IJle(else_label.clone()));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
                    instrs.push(Instr::IJmp(end_label.clone()));
                    instrs.push(Instr::ILabel(else_label));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)));
                    instrs.push(Instr::ILabel(end_label));
                },
                BinOp::LessEq => {
                    let else_label = new_label(label_counter, "lesseq_else");
                    let end_label = new_label(label_counter, "lesseq_end");

                    instrs.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset)));
                    instrs.push(Instr::IJg(else_label.clone()));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
                    instrs.push(Instr::IJmp(end_label.clone()));
                    instrs.push(Instr::ILabel(else_label));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)));
                    instrs.push(Instr::ILabel(end_label));
                },
                BinOp::GreaterEq => {
                    let else_label = new_label(label_counter, "greatereq_else");
                    let end_label = new_label(label_counter, "greatereq_end");

                    instrs.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset)));
                    instrs.push(Instr::IJl(else_label.clone()));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
                    instrs.push(Instr::IJmp(end_label.clone()));
                    instrs.push(Instr::ILabel(else_label));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)));
                    instrs.push(Instr::ILabel(end_label));
                },
                BinOp::Equal => {
                    let else_label = new_label(label_counter, "equal_else");
                    let end_label = new_label(label_counter, "equal_end");

                    instrs.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, offset)));
                    instrs.push(Instr::IJne(else_label.clone()));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(FALSE_VAL)));
                    instrs.push(Instr::IJmp(end_label.clone()));
                    instrs.push(Instr::ILabel(else_label));
                    instrs.push(Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(TRUE_VAL)));
                    instrs.push(Instr::ILabel(end_label));
                },    
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

                instrs.extend(compile_to_instrs(expr, current_si, &current_env, label_counter, break_target));
                instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, -8*current_si), Val::Reg(Reg::RAX)));
                current_env.insert(name.clone(), -8*current_si);
                current_si += 1;
            }

            instrs.extend(compile_to_instrs(body, current_si, &current_env, label_counter, break_target));
            instrs
        }

        Expr::Bool(b) => {
            let val = if *b { 1 } else { 3 };
            vec![Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(val))]
        }

        Expr::If(cond, then_expr, else_expr) => {
            let else_label = new_label(label_counter, "if_else");
            let end_label = new_label(label_counter, "if_end");
            let mut instrs = Vec::new();

            instrs.extend(compile_to_instrs(cond, si, env, label_counter, break_target));

            instrs.push(Instr::ICmp(Val::Reg(Reg::RAX), Val::Imm(1)));
            instrs.push(Instr::IJne(else_label.clone()));

            instrs.extend(compile_to_instrs(then_expr, si, env, label_counter, break_target));
            instrs.push(Instr::IJmp(end_label.clone()));

            instrs.push(Instr::ILabel(else_label));
            instrs.extend(compile_to_instrs(else_expr, si, env, label_counter, break_target));
            instrs.push(Instr::ILabel(end_label));
            instrs

        }

        Expr::Loop(body) => {
            let start_label = new_label(label_counter, "loop_start");
            let end_label = new_label(label_counter, "loop_end");
            let mut instrs = Vec::new();

            instrs.push(Instr::ILabel(start_label.clone()));
            instrs.extend(compile_to_instrs(body, si, env, label_counter, &Some(end_label.clone())));
            instrs.push(Instr::IJmp(start_label));
            instrs.push(Instr::ILabel(end_label));
            instrs
        }

        Expr::Break(expr) => {
            match break_target {
                Some(label) => {
                    let mut instrs = compile_to_instrs(expr, si, env, label_counter, break_target);
                    instrs.push(Instr::IJmp(label.clone()));
                    instrs
                },
                None => panic!("Break statement not within a loop"),
            }
        }

        Expr::Set(name, expr) => {
            let mut instrs = compile_to_instrs(expr, si, env, label_counter, break_target);
            let offset = env.get(name).unwrap_or_else(|| panic!("Unbound variable identifier {}", name));
            instrs.push(Instr::IMov(Val::RegOffset(Reg::RSP, *offset), Val::Reg(Reg::RAX)));
            instrs
        }

        Expr::Input => vec![Instr::IMov(Val::Reg(Reg::RAX), Val::Reg(Reg::RDI))],
        Expr::Var(name) => {
            let offset = env.get(name).unwrap_or_else(|| panic!("Unbound variable identifier {}", name));
            vec![Instr::IMov(Val::Reg(Reg::RAX), Val::RegOffset(Reg::RSP, *offset))]
        }
        Expr::Block(exprs) => {
            let mut instrs = Vec::new();
            for expr in exprs {
                instrs.extend(compile_to_instrs(expr, si, env, label_counter, break_target));
            }
            instrs
        }

        _ => panic!("TODO: Implement compile_to_instrs for {:?}", e),
    }
}

fn new_label(label_counter: &mut i32, name: &str) -> String {
    *label_counter += 1;
    format!("{}_{}", name, label_counter)
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
        extern _snek_error              ; Tell linker snek_error is external
        global our_code_starts_here    ; Add underscore for Mac
        our_code_starts_here:          ; Add underscore for Mac
            {}
            ret
        
        throw_error:                    ; Give this a unique name
            mov rdi, 1                  ; Put error code in rdi
            call _snek_error

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
        Val::Reg(Reg::RBX) => String::from("rbx"),
        Val::Reg(Reg::RDI) => String::from("rdi"),
        Val::Imm(n) => format!("{}", n),
        Val::RegOffset(reg, offset) => {
            let r_str = match reg {
                Reg::RAX => "rax",
                Reg::RSP => "rsp",
                Reg::RBX => "rbx",
                Reg::RDI => "rdi", 
            };
            if *offset < 0 {
                format!("[{} - {}]", r_str, -offset)
            } else {
                format!("[{} + {}]", r_str, offset)
            }
        }

    }
}

/// Convert an Instr to its assembly string representation
fn instr_to_str(i: &Instr) -> String {
    match i {
        Instr::IMov(dst, src) => format!("mov {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::IAdd(dst, src) => format!("add {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::ISub(dst, src) => format!("sub {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::IMul(dst, src) => format!("imul {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::ICmp(lhs, rhs) => format!("cmp {}, {}", val_to_str(lhs), val_to_str(rhs)),
        Instr::IJmp(label) => format!("jmp {}", label),
        Instr::IJne(label) => format!("jne {}", label),
        Instr::ILabel(label) => format!("{}:", label),
        Instr::IAnd(dst, src) => format!("and {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::IJge(label) => format!("jge {}", label),
        Instr::IJle(label) => format!("jle {}", label),
        Instr::IJg(label) => format!("jg {}", label),
        Instr::IJl(label) => format!("jl {}", label),
        Instr::ISar(dst, src) => format!("sar {}, {}", val_to_str(dst), val_to_str(src)),
    }
}

/// Compile an expression to a complete assembly string
fn compile(e: &Expr) -> String {
    let env: HashMap<String, i32> = HashMap::new();
    let mut label_counter = 0;
    let instrs = compile_to_instrs(e, 2, &env, &mut label_counter, &None);
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
