use std::time::{Duration, Instant};
struct Deadline(Instant);
impl fend_core::Interrupt for Deadline {
    fn should_interrupt(&self) -> bool {
        self.0.elapsed() > Duration::from_millis(150) || !crate::modules::enabled("calculator")
    }
}
static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[tauri::command]
pub async fn evaluate_math(expression: String) -> Result<String, String> {
    crate::modules::require("calculator")?;
    if expression.len() > 512 {
        return Err("Keep expressions under 512 bytes.".into());
    }
    if BUSY.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("Calculation busy. Try again.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                BUSY.store(false, std::sync::atomic::Ordering::SeqCst);
            }
        }
        let _guard = Guard;
        let mut context = fend_core::Context::new();
        let result = fend_core::evaluate_with_interrupt(
            &expression,
            &mut context,
            &Deadline(Instant::now()),
        )?;
        let result = result.get_main_result();
        if result.len() > 4096 {
            return Err("Result is too large to display.".into());
        }
        Ok(result.to_owned())
    })
    .await
    .map_err(|e| e.to_string())?
}
