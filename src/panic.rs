use crate::*;

#[panic_handler]
#[allow(unreachable_code)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Panic: ");
    if let Some(location) = info.location() {
        println!(
            "line: {}, file: {}: {}",
            location.line(),
            location.file(),
            info.message().unwrap()
        );
    } else {
        println!("No information available");
    }

    #[cfg(test)]
    {
        println!("test result: FAILED.");
        test::exit_failure();
    }

    loop {}
}
