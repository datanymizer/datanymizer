use indicatif::{ProgressBar, ProgressStyle};

pub trait DumpProgressBar {
    fn new_progress_bar(can_log_to_stdout: bool) -> ProgressBar {
        if can_log_to_stdout {
            ProgressBar::new(0)
        } else {
            ProgressBar::hidden()
        }
    }

    fn progress_bar(&self) -> &ProgressBar;

    fn init_progress_bar(&self, tsize: u64, prefix: &str) {
        let delta = tsize / 100;
        self.progress_bar().set_length(tsize);
        self.progress_bar().set_draw_delta(delta);
        self.progress_bar().set_prefix(prefix);
        self.progress_bar().set_style(
            ProgressStyle::default_bar()
                .template(
                    "[Dumping: {prefix}] [|{bar:50}|] {pos} of {len} rows [{percent}%] ({eta})",
                )
                .progress_chars("#>-"),
        );
    }

    fn inc_progress_bar(&self) {
        self.progress_bar().inc(1);
    }

    fn finish_progress_bar(&self) {
        let pb = self.progress_bar();
        pb.finish();
        pb.reset();
    }
}
