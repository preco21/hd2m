use ndarray::parallel::prelude::*;
use ndarray::s;
use ndarray::Array;
use ndarray::Array1;
use ndarray::Array2;
use ndarray::Axis;
use ndarray::Zip;

fn main() {
    let arr = Array2::from_shape_vec(
        (15, 9),
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            1, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 1, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 1, 0, 0, //
            0, 0, 0, 0, 0, 1, 0, 0, 0, //
            0, 0, 1, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 1, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 1, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ],
    );
    let arr2 = Array2::from_shape_vec(
        (15, 9),
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            2, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 2, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 2, 2, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 2, 0, //
            0, 2, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 2, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 2, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 2, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ],
    );

    println!("Total: {:?}", arr);

    let mut a = Array2::<i32>::zeros((15, 9));

    Zip::from(&mut a)
        .and(&arr)
        .and(&arr2)
        .par_for_each(|a, &b| {
            *a = b;
        });

    for row in arr.axis_iter(Axis(0)) {
        println!("Row: {:?}", row);
    }

    println!("Total: {:?}", arr);

    // // Traverse the array with a window size of 3 along axis 1
    // for (window, correct) in arr.axis_windows(Axis(0), 3).into_iter().zip(&correct) {
    //     println!("Window: {:?}", window);
    //     // assert_eq!(window, correct);
    //     // assert_eq!(window.shape(), &[4, 3, 2]);
    // }
}
