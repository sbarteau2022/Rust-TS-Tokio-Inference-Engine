#include <os/log.h>
#include <os/signpost.h>
#include <cstdint>
static os_log_t job_log() {
    static os_log_t log = os_log_create("engineering.atoma.rust-server", "jobs");
    return log;
}
extern "C" uint64_t rs_span_begin(uint64_t job) {
    auto log = job_log();
    auto id = os_signpost_id_generate(log);
    os_signpost_interval_begin(log, id, "Kernel Job", "job=%{public}llu", (unsigned long long)job);
    os_log_with_type(log, OS_LOG_TYPE_DEFAULT, "job_begin job=%{public}llu span=%{public}llu", (unsigned long long)job, (unsigned long long)id);
    return id;
}
extern "C" void rs_span_end(uint64_t id, uint64_t job, bool success) {
    auto log = job_log();
    os_signpost_interval_end(log, id, "Kernel Job", "job=%{public}llu success=%{public}d", (unsigned long long)job, success);
    os_log_with_type(log, OS_LOG_TYPE_DEFAULT, "job_end job=%{public}llu span=%{public}llu success=%{public}d", (unsigned long long)job, (unsigned long long)id, success);
}
extern "C" bool rs_span_enabled() { return os_signpost_enabled(job_log()); }

#include <unistd.h>
extern "C" uint64_t rs_host_page_size() { return (uint64_t)getpagesize(); }
