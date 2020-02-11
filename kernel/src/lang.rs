use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n*****PANIC*****\nOh! There has some problem!\n{}", info);

    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort!");
}

#[lang = "eh_personality"]
#[no_mangle]
fn eh_personality() -> ! {
    loop {}
}