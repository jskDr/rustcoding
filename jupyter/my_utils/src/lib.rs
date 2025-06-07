// my-utils/src/lib.rs

pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to the local library.", name)
}

pub fn slow_calculation() {
    // 일부러 시간을 소요하는 복잡한 계산이 있다고 가정
    std::thread::sleep(std::time::Duration::from_secs(2));
    println!("Slow calculation finished!");
}