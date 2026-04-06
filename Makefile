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
test: test/simple.run test/add.run test/binop.run test/let_simple.run test/let_multi.run test/nested.run test/simple_loop.run test/simple_comparison.run test/nested_if.run test/compare_equal.run test/compare_not_equal.run test/compare_less_than.run test/compare_gt_equal.run test/descending_loop.run
	@echo "Running tests..."
	@echo -n "simple (expected 42): " && ./test/simple.run
	@echo -n "add (expected 6): " && ./test/add.run
	@echo -n "binop (expected 9): " && ./test/binop.run
	@echo -n "let_simple (expected 11): " && ./test/let_simple.run
	@echo -n "let_multi (expected 11): " && ./test/let_multi.run
	@echo -n "nested (expected 25): " && ./test/nested.run
	@echo -n "simple loop (expected 10): " && ./test/simple_loop.run
	@echo -n "simple comparison (expected true): " && ./test/simple_comparison.run
	@echo -n "nested if (expected 100): " && ./test/nested_if.run
	@echo -n "compare equal (expected true): " && ./test/compare_equal.run
	@echo -n "compare not equal (expected false): " && ./test/compare_not_equal.run
	@echo -n "compare less than (expected true): " && ./test/compare_less_than.run
	@echo -n "compare greater than or equal (expected true): " && ./test/compare_gt_equal.run
	@echo -n "descending loop (expected 0): " && ./test/descending_loop.run


.PHONY: clean test
