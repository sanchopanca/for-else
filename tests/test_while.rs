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
