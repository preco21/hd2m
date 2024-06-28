use opencv::{
    core::{self, bitwise_not, Mat, Point2f, Rect, Scalar, Size, Vector},
    features2d::{BFMatcher, SIFT},
    imgcodecs::{imread, imwrite, IMREAD_COLOR},
    imgproc::{self, cvt_color, COLOR_BGR2GRAY, COLOR_BGR2RGB, LINE_AA},
    prelude::*,
    types::{
        VectorOfDMatch, VectorOfKeyPoint, VectorOfMat, VectorOfPoint2f, VectorOfVectorOfDMatch,
    },
};

fn find_matching_boxes(
    image: &Mat,
    template: &Mat,
    max_matching_objects: i32,
    sift_distance_threshold: f32,
    best_matches_points: usize,
) -> Result<Vec<VectorOfPoint2f>, Box<dyn std::error::Error>> {
    // Initialize the detector and matcher
    let detector = SIFT::create(0, 3, 0.04, 10.0, 1.6, true)?;
    let bf = BFMatcher::new(core::NORM_L2, false)?;

    // Find keypoints and descriptors for the template
    let mut keypoints_template = VectorOfKeyPoint::new();
    let mut descriptors_template = Mat::default();
    detector.detect_and_compute(
        &template,
        &Mat::default(),
        &mut keypoints_template,
        &mut descriptors_template,
        false,
    )?;

    let mut matched_boxes = Vec::new();
    let mut matching_img = image.clone();

    for _ in 0..max_matching_objects {
        // Find keypoints and descriptors for the matching image
        let mut keypoints_image = VectorOfKeyPoint::new();
        let mut descriptors_image = Mat::default();
        detector.detect_and_compute(
            &matching_img,
            &Mat::default(),
            &mut keypoints_image,
            &mut descriptors_image,
            false,
        )?;

        // Match descriptors
        let mut matches = VectorOfVectorOfDMatch::new();
        bf.knn_train_match(
            &descriptors_image,
            &descriptors_template,
            &mut matches,
            2,
            &Mat::default(),
            false,
        )?;

        // Filter good matches
        let good_matches: VectorOfDMatch = matches
            .iter()
            .filter_map(|m| {
                if m.len() == 2
                    && m.get(0).unwrap().distance
                        < sift_distance_threshold * m.get(1).unwrap().distance
                {
                    Some(m.get(0).unwrap())
                } else {
                    None
                }
            })
            .take(best_matches_points)
            .collect();

        if good_matches.is_empty() {
            break;
        }

        // Extract location of good matches
        let points_image: VectorOfPoint2f = good_matches
            .iter()
            .map(|m| keypoints_image.get(m.query_idx as usize).unwrap().pt())
            .collect();
        let points_template: VectorOfPoint2f = good_matches
            .iter()
            .map(|m| keypoints_template.get(m.train_idx as usize).unwrap().pt())
            .collect();

        // Find homography
        // let h = match core::find_homography(&points_template, &points_image, RANSAC, 2.0, &mut Mat::default(), 200, 0.995) {
        //     Ok(h) => h,
        //     Err(_) => break,
        // };

        // Transform the corners of the template to the matching points in the image
        let (h, w) = (template.rows(), template.cols());
        let corners = VectorOfPoint2f::from(vec![
            Point2f::new(0.0, 0.0),
            Point2f::new(0.0, h as f32),
            Point2f::new(w as f32, h as f32),
            Point2f::new(w as f32, 0.0),
        ]);
        let mut transformed_corners = VectorOfPoint2f::new();
        // core::perspective_transform(&corners, &mut transformed_corners, &h)?;

        matched_boxes.push(transformed_corners);

        // Create a mask and fill the matched area
        let mut mask = Mat::ones(matching_img.size()?, core::CV_8U)?;
        imgproc::fill_poly(
            &mut mask,
            &matched_boxes,
            Scalar::all(0.0),
            LINE_AA,
            0,
            core::Point::default(),
        )?;
        bitwise_not(&mask, &mut mask)?;
        core::bitwise_and(&matching_img, &matching_img, &mut matching_img, &mask)?;
    }

    Ok(matched_boxes)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image = imread("i_remoter.png", IMREAD_COLOR)?;
    let template = imread("t_remoter.png", IMREAD_COLOR)?;

    let max_matching_objects = 10;
    let sift_distance_threshold = 0.85;
    let best_matches_points = 500;

    let matched_boxes = find_matching_boxes(
        &image,
        &template,
        max_matching_objects,
        sift_distance_threshold,
        best_matches_points,
    )?;

    let mut output_image = image.clone();
    for b in matched_boxes {
        // let pts = VectorOfMat::from_iter(b.iter());
        imgproc::polylines(
            &mut output_image,
            &b,
            true,
            Scalar::new(0.0, 255.0, 0.0, 0.0),
            3,
            LINE_AA,
            0,
        )?;
    }

    imwrite("result.png", &output_image, &Vector::new())?;

    Ok(())
}
