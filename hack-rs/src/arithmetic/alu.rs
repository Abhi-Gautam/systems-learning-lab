//! # Arithmetic Logic Unit (ALU)
//!
//! The ALU is the computational heart of the Hack CPU.
//! It takes two 16-bit inputs and 6 control bits, producing a 16-bit output.
//!
//! ## Control Bits
//!
//! ```text
//! zx: Zero the x input
//! nx: Negate the x input (after zeroing)
//! zy: Zero the y input
//! ny: Negate the y input (after zeroing)
//! f:  Function select (0 = AND, 1 = ADD)
//! no: Negate the output
//! ```
//!
//! ## How It Works
//!
//! The ALU processes inputs through a pipeline:
//!
//! ```text
//!        ┌─────┐     ┌─────┐
//! x ────►│ zx  ├────►│ nx  ├────┐
//!        └─────┘     └─────┘    │
//!                               ▼
//!                          ┌─────────┐     ┌─────┐
//!                          │  f=0:AND├────►│ no  ├────► out
//!                          │  f=1:ADD│     └─────┘
//!                          └─────────┘
//!                               ▲
//!        ┌─────┐     ┌─────┐    │
//! y ────►│ zy  ├────►│ ny  ├────┘
//!        └─────┘     └─────┘
//! ```
//!
//! ## The 18 Operations
//!
//! By setting the 6 control bits, you get 18 useful operations:
//!
//! ```text
//! zx nx zy ny  f no | out
//! -------------------+----------------
//!  1  0  1  0  1  0  | 0
//!  1  1  1  1  1  1  | 1
//!  1  1  1  0  1  0  | -1
//!  0  0  1  1  0  0  | x
//!  1  1  0  0  0  0  | y
//!  0  0  1  1  0  1  | !x
//!  1  1  0  0  0  1  | !y
//!  0  0  1  1  1  1  | -x
//!  1  1  0  0  1  1  | -y
//!  0  1  1  1  1  1  | x+1
//!  1  1  0  1  1  1  | y+1
//!  0  0  1  1  1  0  | x-1
//!  1  1  0  0  1  0  | y-1
//!  0  0  0  0  1  0  | x+y
//!  0  1  0  0  1  1  | x-y
//!  0  0  0  1  1  1  | y-x
//!  0  0  0  0  0  0  | x&y
//!  0  1  0  1  0  1  | x|y
//! ```

use crate::gates::{mux16, not16, and16, or8way};
use super::adder::add16;

/// ALU output including status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AluOutput {
    /// The 16-bit result
    pub out: u16,
    /// Zero flag: true if out == 0
    pub zr: bool,
    /// Negative flag: true if out < 0 (i.e., bit 15 is 1)
    pub ng: bool,
}

/// The Hack ALU - computes one of 18 functions based on control bits.
///
/// # Arguments
///
/// * `x` - First 16-bit input
/// * `y` - Second 16-bit input
/// * `zx` - Zero the x input
/// * `nx` - Negate x (after zx)
/// * `zy` - Zero the y input
/// * `ny` - Negate y (after zy)
/// * `f` - Function: 0 = AND, 1 = ADD
/// * `no` - Negate the output
///
/// # Returns
///
/// `AluOutput` containing:
/// - `out`: The 16-bit result
/// - `zr`: True if result is zero
/// - `ng`: True if result is negative (two's complement)
///
/// # Implementation Steps
///
/// 1. If zx, set x = 0
/// 2. If nx, set x = !x
/// 3. If zy, set y = 0
/// 4. If ny, set y = !y
/// 5. If f, out = x + y, else out = x & y
/// 6. If no, out = !out
/// 7. Compute zr and ng from out
pub fn alu(
    x: u16,
    y: u16,
    zx: bool,
    nx: bool,
    zy: bool,
    ny: bool,
    f: bool,
    no: bool,
) -> AluOutput {
    todo!("Implement the ALU following the steps in the docstring")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to make tests more readable
    fn run_alu(x: u16, y: u16, zx: u8, nx: u8, zy: u8, ny: u8, f: u8, no: u8) -> AluOutput {
        alu(x, y, zx == 1, nx == 1, zy == 1, ny == 1, f == 1, no == 1)
    }

    #[test]
    fn test_alu_zero() {
        // zx=1, nx=0, zy=1, ny=0, f=1, no=0 → 0
        let result = run_alu(0x1234, 0x5678, 1, 0, 1, 0, 1, 0);
        assert_eq!(result.out, 0);
        assert!(result.zr);
        assert!(!result.ng);
    }

    #[test]
    fn test_alu_one() {
        // zx=1, nx=1, zy=1, ny=1, f=1, no=1 → 1
        let result = run_alu(0x1234, 0x5678, 1, 1, 1, 1, 1, 1);
        assert_eq!(result.out, 1);
        assert!(!result.zr);
        assert!(!result.ng);
    }

    #[test]
    fn test_alu_minus_one() {
        // zx=1, nx=1, zy=1, ny=0, f=1, no=0 → -1 (0xFFFF)
        let result = run_alu(0x1234, 0x5678, 1, 1, 1, 0, 1, 0);
        assert_eq!(result.out, 0xFFFF);
        assert!(!result.zr);
        assert!(result.ng);
    }

    #[test]
    fn test_alu_x() {
        // zx=0, nx=0, zy=1, ny=1, f=0, no=0 → x
        let result = run_alu(0x1234, 0x5678, 0, 0, 1, 1, 0, 0);
        assert_eq!(result.out, 0x1234);
    }

    #[test]
    fn test_alu_y() {
        // zx=1, nx=1, zy=0, ny=0, f=0, no=0 → y
        let result = run_alu(0x1234, 0x5678, 1, 1, 0, 0, 0, 0);
        assert_eq!(result.out, 0x5678);
    }

    #[test]
    fn test_alu_not_x() {
        // zx=0, nx=0, zy=1, ny=1, f=0, no=1 → !x
        let result = run_alu(0x00FF, 0x5678, 0, 0, 1, 1, 0, 1);
        assert_eq!(result.out, 0xFF00);
    }

    #[test]
    fn test_alu_not_y() {
        // zx=1, nx=1, zy=0, ny=0, f=0, no=1 → !y
        let result = run_alu(0x1234, 0x00FF, 1, 1, 0, 0, 0, 1);
        assert_eq!(result.out, 0xFF00);
    }

    #[test]
    fn test_alu_neg_x() {
        // zx=0, nx=0, zy=1, ny=1, f=1, no=1 → -x
        // -x in two's complement = !x + 1
        let result = run_alu(5, 0x5678, 0, 0, 1, 1, 1, 1);
        // -5 = 0xFFFB
        assert_eq!(result.out, 0xFFFB);
        assert!(result.ng);
    }

    #[test]
    fn test_alu_neg_y() {
        // zx=1, nx=1, zy=0, ny=0, f=1, no=1 → -y
        let result = run_alu(0x1234, 5, 1, 1, 0, 0, 1, 1);
        assert_eq!(result.out, 0xFFFB); // -5
        assert!(result.ng);
    }

    #[test]
    fn test_alu_x_plus_1() {
        // zx=0, nx=1, zy=1, ny=1, f=1, no=1 → x+1
        let result = run_alu(10, 0x5678, 0, 1, 1, 1, 1, 1);
        assert_eq!(result.out, 11);
    }

    #[test]
    fn test_alu_y_plus_1() {
        // zx=1, nx=1, zy=0, ny=1, f=1, no=1 → y+1
        let result = run_alu(0x1234, 10, 1, 1, 0, 1, 1, 1);
        assert_eq!(result.out, 11);
    }

    #[test]
    fn test_alu_x_minus_1() {
        // zx=0, nx=0, zy=1, ny=1, f=1, no=0 → x-1
        let result = run_alu(10, 0x5678, 0, 0, 1, 1, 1, 0);
        assert_eq!(result.out, 9);
    }

    #[test]
    fn test_alu_y_minus_1() {
        // zx=1, nx=1, zy=0, ny=0, f=1, no=0 → y-1
        let result = run_alu(0x1234, 10, 1, 1, 0, 0, 1, 0);
        assert_eq!(result.out, 9);
    }

    #[test]
    fn test_alu_x_plus_y() {
        // zx=0, nx=0, zy=0, ny=0, f=1, no=0 → x+y
        let result = run_alu(5, 3, 0, 0, 0, 0, 1, 0);
        assert_eq!(result.out, 8);
        assert!(!result.zr);
        assert!(!result.ng);
    }

    #[test]
    fn test_alu_x_minus_y() {
        // zx=0, nx=1, zy=0, ny=0, f=1, no=1 → x-y
        let result = run_alu(10, 3, 0, 1, 0, 0, 1, 1);
        assert_eq!(result.out, 7);
    }

    #[test]
    fn test_alu_y_minus_x() {
        // zx=0, nx=0, zy=0, ny=1, f=1, no=1 → y-x
        let result = run_alu(3, 10, 0, 0, 0, 1, 1, 1);
        assert_eq!(result.out, 7);
    }

    #[test]
    fn test_alu_x_and_y() {
        // zx=0, nx=0, zy=0, ny=0, f=0, no=0 → x&y
        let result = run_alu(0xFF00, 0x0FF0, 0, 0, 0, 0, 0, 0);
        assert_eq!(result.out, 0x0F00);
    }

    #[test]
    fn test_alu_x_or_y() {
        // zx=0, nx=1, zy=0, ny=1, f=0, no=1 → x|y
        // De Morgan: x|y = !(!x & !y)
        let result = run_alu(0xFF00, 0x0FF0, 0, 1, 0, 1, 0, 1);
        assert_eq!(result.out, 0xFFF0);
    }

    #[test]
    fn test_alu_flags() {
        // Test zero flag
        let result = run_alu(5, 5, 0, 1, 0, 0, 1, 1); // x - y = 0
        assert!(result.zr);
        assert!(!result.ng);

        // Test negative flag
        let result = run_alu(3, 10, 0, 1, 0, 0, 1, 1); // x - y = -7
        assert!(!result.zr);
        assert!(result.ng);
    }
}
