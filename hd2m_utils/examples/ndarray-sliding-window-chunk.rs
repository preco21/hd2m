use ndarray::parallel::prelude::*;
use ndarray::s;
use ndarray::Array;
use ndarray::Array2;
use ndarray::Axis;
use ndarray::IntoNdProducer;

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

    // let res = arr
    //     .axis_windows(Axis(0), 3)
    //     .into_iter()
    //     .map(|rows| rows.map_axis(Axis(1), |col| col.last().unwrap()).to_owned())
    //     .collect::<Vec<Vec<_>>>();

    // arr.axis_windows(Axis(0), 3)
    //     .into_iter()
    //     .map(|rows| rows.windows((Axis(1), vec![], |acc, el| el));

    // 히스토그램 이하 코드는 굳이 matrix 유지할 필요 없음

    // arr.windows((Axis(0), 3))
    //     .into_iter()
    //     .map(|rows| rows.map_axis(Axis(1), |col| col.last().unwrap()).to_owned())
    //     .collect::<Vec<Array2<_>>>();

    let start = std::time::Instant::now();
    let res = arr
        .axis_windows(Axis(0), 3)
        .into_iter()
        // .map(|rows| rows.into_par_iter().find_first(|&&col| col > 0))
        .map(|rows| {
            rows.axis_iter(Axis(1))
                .into_par_iter()
                .map(|col| col.into_par_iter().find_first(|&&el| el > 0).copied())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    println!("Elapsed: {:?}", start.elapsed());
    println!("Result: {:?}", res);
}
