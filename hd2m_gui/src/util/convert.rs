use anyhow::Result;
use opencv as cv;
use std::os::raw::c_void;

pub fn convert_frame_to_mat(from: &mut windows_capture::frame::Frame) -> Result<cv::core::Mat> {
    let mut data = from.buffer()?;
    let mat = unsafe {
        cv::core::Mat::new_rows_cols_with_data_unsafe(
            data.height() as i32,
            data.width() as i32,
            cv::core::CV_8UC4,
            data.as_raw_nopadding_buffer()?.as_ptr() as *mut c_void,
            cv::core::Mat_AUTO_STEP,
        )?
    };
    Ok(mat)
}
