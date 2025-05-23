use for_else::for_;

#[test]
fn test_if_block() {
    let mut flag = true;
    for_! { i in 0..10 {
        if i == 5 {
            break;
        }
    } else {
        flag = false;
    }}

    // else_! {
    //     flag = false;
    // }

    assert!(flag);
}

#[test]
fn test_else_block() {
    let mut flag = true;
    for_! { i in 0..10 {
        if i < 5 {
        } else {
            break;
        }
    } else {
        flag = false;
    }}

    assert!(flag);
}

#[test]
fn test_match_arm_statemnt() {
    let mut flag = true;
    for_! { i in 0..10 {
        match i {
            5 => break,
            _ => {}
        };
    } else {
        flag = false;
    }}

    assert!(flag);
}

#[test]
fn test_match_arm_block() {
    let mut flag = true;
    for_! { i in 0..10 {
        match i {
            5..=6 => {
                println!();
                break
            },
            9 => break,
            _ => {}
        };
    } else {
        flag = false;
    }}

    assert!(flag);
}

#[test]
#[allow(clippy::collapsible_if)]
fn test_deep() {
    let mut flag = true;
    for_! { i in 0..10 {
        if i > 2  {
            if i > 3 {
                if i > 4 {
                    if i > 5 {
                        if i > 6 {
                            if i > 7 {
                                if i > 8 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        flag = false;
    }}

    assert!(flag);
}

#[test]
fn test_negative() {
    let mut flag = false;
    for_! { i in 0..10 {
        if i == 11 {
            break;
        }
    } else {
        flag = true;
    }}

    assert!(flag);
}

#[test]
fn test_indent() {
    let mut was_in_else_branch = false;
    let named_expr = [1, 2, 3, 4, 5];
    for_! { i in named_expr {
        if i == 3 {
            break;
        }
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
fn test_indent_else() {
    let mut was_in_else_branch = false;
    let named_expr = [1, 2, 3, 4, 5];
    for_! { i in named_expr {
        if i == 8 {
            break;
        }
    } else {
        was_in_else_branch = true;
    }}

    assert!(was_in_else_branch);
}

struct S;
impl S {
    fn iter(self) -> std::vec::IntoIter<u8> {
        vec![1, 2, 3].into_iter()
    }
}

#[test]
fn test_inline_struct() {
    let mut was_in_else_branch = false;
    for_! { i in (S {}).iter() {
        if i == 2 {
            break;
        }
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
fn test_inline_struct_else() {
    let mut was_in_else_branch = false;
    for_! { i in (S {}).iter() {
        if i == 10 {
            break;
        }
    } else {
        was_in_else_branch = true;
    }}

    assert!(was_in_else_branch);
}

#[test]
fn test_inline_block() {
    let mut was_in_else_branch = false;
    for_! { i in { vec![1, 2, 3, 4, 5] }.into_iter() {
        if i == 3 {
            break;
        }
    } else {
        was_in_else_branch = true;
    }}

    assert!(!was_in_else_branch);
}

#[test]
fn test_inline_block_with_inline_struct_else() {
    let mut was_in_else_branch = false;
    for_! { i in { S {} }.iter() {
        if i == 10 {
            break;
        }
    } else {
        was_in_else_branch = true;
    }}
    assert!(was_in_else_branch);
}
