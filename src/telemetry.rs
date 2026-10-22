unsafe extern "C" {
    fn rs_span_begin(job: u64) -> u64;
    fn rs_span_end(id: u64, job: u64, success: bool);
    fn rs_span_enabled() -> bool;
}
pub struct Span {
    pub id: u64,
    pub enabled: bool,
    job: u64,
    pub success: bool,
}
impl Span {
    pub fn begin(job: u64) -> Self {
        Self {
            id: unsafe { rs_span_begin(job) },
            enabled: unsafe { rs_span_enabled() },
            job,
            success: false,
        }
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        unsafe { rs_span_end(self.id, self.job, self.success) }
    }
}
