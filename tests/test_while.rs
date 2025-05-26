use for_else::while_;

#[test]
fn test_while() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    while_! { x < 10 {
        if x == 5 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
fn test_while_else() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    while_! { x < 10 {
        if x < 0 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(was_in_else_branch);
}

#[test]
fn test_while_var_in_last_position() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    let limit = 10;
    while_! { x < limit {
        if x == 5 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
fn test_while_var_in_last_position_else() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    let limit = 10;
    while_! { x < limit {
        if x < 0 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(was_in_else_branch);
}

struct S {}

impl S {
    fn cond(self, x: i32) -> bool {
        x < 10
    }
}

#[test]
fn test_while_inline_struct() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    while_! { (S {}).cond(x) {
        if x == 5 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
fn test_while_inline_struct_else() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    while_! { (S {}).cond(x) {
        if x < 0 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(was_in_else_branch);
}

#[test]
fn test_while_block_expr() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    while_! { { let s = S {}; s.cond(x) } {
        if x == 5 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
fn test_while_block_expr_else() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    while_! { { let s = S {}; s.cond(x) } {
        if x < 0 {
            break;
        }
        x += 1;
    } else {
        was_in_else_branch = true;
    }}

    assert!(was_in_else_branch);
}

#[test]
#[allow(clippy::never_loop)]
fn test_while_with_label_with_loop_inside() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    let limit = 10;
    while_! { 'outer: x < limit {
        loop {
            if x == 5 {
                break 'outer;
            }
            x += 1;
        }
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
#[allow(unused_labels)]
fn test_while_with_label_with_loop_inside_else() {
    let mut was_in_else_branch = false;
    let mut x = 0;
    let limit = 10;
    while_! { 'outer: x < limit {
        loop {
            x += 1;
            if x > 5 {
                break;
            }
        }
    } else {
        was_in_else_branch = true;
    }}

    assert!(was_in_else_branch);
}
