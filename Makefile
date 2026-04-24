.PRECIOUS: test/%.s
# Makefile for Boa Compiler
# Replace 'elf64' with 'macho64' on macOS

# Pattern rule to compile .snek files to .s assembly files
test/%.s: test/%.snek src/main.rs
	cargo run -- $< test/$*.s

# Pattern rule to assemble .s files and link into executables
# Change -f elf64 to -f macho64 on macOS
test/%.run: test/%.s runtime/start.rs
	nasm -f macho64 test/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc --target x86_64-apple-darwin -L runtime/ runtime/start.rs -o test/$*.run

# Clean build artifacts
clean:
	cargo clean
	rm -f test/*.s test/*.run runtime/*.o runtime/*.a

# Run all tests
test: test/zero_arg.run test/one_arg.run test/two_arg_add.run test/five_arg_sum.run test/six_arg_sum.run test/factorial_recursive.run test/factorial_iterative.run test/fib_rec.run test/power_func.run test/gcd_func.run test/triangle_num.run test/countdown.run test/mutual_rec.run test/more_mutual_rec.run test/helper_function.run test/triple_call.run test/max_out_of_three.run test/let_in_func.run test/multiple_lets_in_func.run test/shadow_param_func.run test/mixed_param_local.run test/local_in_rec.run test/multiple_args_locals.run 
	@echo "Running tests..."
	@echo -n "zero arguments (expected 5): " && ./test/zero_arg.run
	@echo -n "one argument (expected 42): " && ./test/one_arg.run
	@echo -n "two arguments (expected 30): " && ./test/two_arg_add.run
	@echo -n "five arguments (expected 15): " && ./test/five_arg_sum.run
	@echo -n "six arguments (expected 6): " && ./test/six_arg_sum.run
	@echo -n "factorial recursive (expected 120): " && ./test/factorial_recursive.run
	@echo -n "factorial iterative (expected 120): " && ./test/factorial_iterative.run
	@echo -n "fibonacci recursive (expected 8): " && ./test/fib_rec.run
	@echo -n "power function (expected 1024): " && ./test/power_func.run
	@echo -n "gcd function (expected 6): " && ./test/gcd_func.run
	@echo -n "triangle numbers (expected 55): " && ./test/triangle_num.run
	@echo -n "countdown function" (expected 0): " && ./test/countdown.run
	@echo -n "mutual recursion (expected true): " && ./test/mutual_rec.run
	@echo -n "more_mutual_rec (expected true): " && ./test/more_mutual_rec.run
	@echo -n "helper function (expected 25): " && ./test/helper_function.run
	@echo -n "triple call (expected 12): " && ./test/triple_call.run
	@echo -n "max_out_of_three (expected 20): " && ./test/max_out_of_three.run
	@echo -n "let in functions (expected 15): " && ./test/let_in_func.run
	@echo -n "multiple let in functions (expected 13): " && ./test/multiple_lets_in_func.run
	@echo -n "shadow parameters (expected 100): " && ./test/shadow_param_func.run
	@echo -n "mixed parameters (expected 20): " && ./test/mixed_param_local.run
	@echo -n "local in recursion (expected 120): " && ./test/local_in_rec.run
	@echo -n "multiple args and locals (expected 25): " && ./test/multiple_args_locals.run




.PHONY: clean test
