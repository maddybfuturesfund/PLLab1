#[link(name = "our_code")]
extern "C" {
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here() -> i64;
}

#[no_mangle]
extern "C" fn snek_error(errcode: i64) {
    if errcode == 1 {
        eprintln!("invalid argument");
    } else if errcode == 2 {
        eprintln!("overflow");
    }
    std::process::exit(1);
}

#[no_mangle]
extern "C" fn snek_print(val: i64) -> i64 {
    if val & 1 == 0 {
        println!("{}", val >> 1);  // Number
    } else if val == 3 {
        println!("true");
    } else if val == 1 {
        println!("false");
    }
    val
}

fn main() {
    let i: i64 = unsafe {
        our_code_starts_here()
    };
    if i % 2 == 0 {
        println!("{}", i >> 1);
    } else if i == 3 {
        println!("true");
    } else if i == 1 {
        println!("false");
    } else {
        println!("Unknown: {}", i);
    }
    //println!("{i}");
}
