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
test: test/fibonacci.run
	@echo "Running tests..."
	@echo -n "fibonacci (expected 55): " && ./test/fibonacci.run


.PHONY: clean test
