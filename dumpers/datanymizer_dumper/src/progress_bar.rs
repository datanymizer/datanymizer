use indicatif::{ProgressBar, ProgressStyle};

pub trait DumpProgressBar {
    fn new_progress_bar(can_log_to_stdout: bool) -> ProgressBar {
        if can_log_to_stdout {
            let pb = ProgressBar::new(0);
            pb.finish_and_clear();
            pb
        } else {
            ProgressBar::hidden()
        }
    }

    fn progress_bar(&self) -> &ProgressBar;

    fn init_progress_bar(&self, tsize: u64, prefix: &str) {
        let delta = tsize / 100;
        let pb = self.progress_bar();
        pb.set_draw_delta(delta);
        pb.set_prefix(prefix);
        pb.set_length(tsize);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "[Dumping: {prefix}] [|{bar:50}|] {pos} of {len} rows [{percent}%] ({eta})",
                )
                .progress_chars("#>-"),
        );
        pb.reset();
    }

    fn inc_progress_bar(&self) {
        self.progress_bar().inc(1);
    }

    fn finish_progress_bar(&self) {
        self.progress_bar().finish();
    }
}
