use super::TryFromCv;
use opencv::core as cv_core;
use std::os::raw::c_void;

impl<'a, 'b> TryFromCv<&'a mut windows_capture::frame::Frame<'b>> for cv_core::Mat {
    type Error = anyhow::Error;

    fn try_from_cv(from: &'a mut windows_capture::frame::Frame<'b>) -> Result<Self, Self::Error> {
        let mut data = from.buffer()?;
        let mat = unsafe {
            cv_core::Mat::new_rows_cols_with_data(
                data.height() as i32,
                data.width() as i32,
                cv_core::CV_8UC4,
                data.as_raw_nopadding_buffer()?.as_ptr() as *mut c_void,
                cv_core::Mat_AUTO_STEP,
            )?
        };
        Ok(mat)
    }
}
