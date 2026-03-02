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
# * also add support for rotating logs, AND create a debugging, monitoring, performance, and optimization system for the kogi os
# * so the "CPUs" that run the kogi os should be independent workers (e.g. contractors, freelancers, consultants, gig workers, etc...), so the entire kogi os app exists to support independent workers in their work endeavors, where all of their work is encapsulated within a "portfolio". refactor the entire app to reflect this design, as well as adding abstractions for ICPUs (independent worker CPUs), IDDs (independent worker device drivers) and IPs (independent worker portfolios), and add all of the necessary supporting code to make these abstractions and systems fully functional accordingly. Also add a calender, scheduling, time+clock management, and profiling systems to the kogi os, also supporting the independent worker abtractions. The entire kogi os app should be cenetered around managing, administrering, maintaining and cotnrolling the independent worker's work (programs, projects, tasks, gigs, contracts, works, artifacts, assets, investments, entites, businesses, , artwork, music, code, hobbies, etc...) portfolio. Update all of the files in the project according to this design paradigm
# * add a caching and register / tiered memory system to the kogi os, and refactor all of the code to utilize this caching and tiered memory system for all data storage and access operations, including the independent worker portfolios, device drivers, CPU scheduling, etc... also add support for memory management, garbage collection, and memory leak detection within this caching and tiered memory system. Update all of the files in the project according to this design paradigm

