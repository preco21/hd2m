use ndarray::s;
use ndarray::Array2;
use ndarray::Axis;

fn main() {
    let arr = Array2::from_shape_fn([4, 5], |(i, j)| i * 100 + j * 10);
    let correct = vec![
        arr.slice(s![.., 0..3]),
        arr.slice(s![.., 1..4]),
        arr.slice(s![.., 2..5]),
    ];

    println!("Total: {:?}", arr);

    // Traverse the array with a window size of 3 along axis 1
    for (window, correct) in arr.axis_windows(Axis(0), 3).into_iter().zip(&correct) {
        println!("Window: {:?}", window);
        // assert_eq!(window, correct);
        // assert_eq!(window.shape(), &[4, 3, 2]);
    }
}
