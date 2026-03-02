# temp script commands
zig build 2>&1 | head -30
zig build test 2>&1 | head -30
zig build test --test-filter "simple test" 2>&1 | head -30
zig build test --test-filter "simple test" --verbose 2>&1 | head -30
zig build test --test-filter "simple test" --verbose 2>&1 | tail -30
