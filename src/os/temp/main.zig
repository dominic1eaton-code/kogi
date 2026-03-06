//! Main entry point for the application.
//! @license MIT
//! @copyright 2026-present Wolof.io Software Studios, Inc.
//! @author Wolof.io Software Studios, Inc.
//! @version 1.0.0
const std = @import("std");
const kogi = @import("kogi");

pub fn main() !void {
    try kogi.run_system();
}
