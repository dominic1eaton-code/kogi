# temp script commands
zig build 2>&1 | head -30
zig build test 2>&1 | head -30
zig build test --test-filter "simple test" 2>&1 | head -30
zig build test --test-filter "simple test" --verbose 2>&1 | head -30
zig build test --test-filter "simple test" --verbose 2>&1 | tail -30


zig build 2>&1 | Select-Object -First 50
zig build test 2>&1 | Select-Object -First 50
zig build test --test-filter "simple test" 2>&1 | Select-Object -First 50
zig build test --test-filter "simple test" --verbose 2>&1 | Select-Object -First 50
zig build test --test-filter "simple test" --verbose 2>&1 | Select-Object -Last 50

zig build 2>&1 | findstr "error:"
zig build run 2>&1 | findstr "leak"

zig build test 2>&1 | Select-Object -First 100
zig build test; $LASTEXITCODE
grep -E "^test \"" src\tests.zig



# notes
# * kogi portfolio management app
# * kogi app should support a wide variety of independent workers, for example: contractors, consultants, freelancers, gig workers, people working on projects, artists, musicians, software developers, gamers, and anyone wanting to organize all of their work, projects, etc... into a single cohesize, indexable, filterable, searchable, manageable portfolio
