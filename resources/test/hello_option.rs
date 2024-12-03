fn check(x: u32) -> Option<bool> {
    if x > 10 {
        Some(true)
    } else {
        None
    }
}

#[cfg(kani)]
#[kani::proof]
fn verify_success() {
    let x: u32 = kani::any();
    check(x).unwrap();
}
