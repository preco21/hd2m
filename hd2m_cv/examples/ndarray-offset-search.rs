use std::default;

use ndarray::parallel::prelude::*;
use ndarray::s;
use ndarray::Array;
use ndarray::Array1;
use ndarray::Array2;
use ndarray::Axis;
use ndarray::Zip;

fn main() -> anyhow::Result<()> {
    let now = std::time::Instant::now();

    let arr = Array2::<i32>::from_shape_vec(
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
    )?;
    let arr2 = Array2::<i32>::from_shape_vec(
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
    )?;
    let arr3 = Array2::<i32>::from_shape_vec(
        (15, 9),
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            3, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 3, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 3, 3, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 3, 3, //
            0, 3, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 3, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 3, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 4, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 3, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ],
    )?;
    let arr4 = Array2::<i32>::from_shape_vec(
        (15, 9),
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            4, 0, 0, 4, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 4, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 4, 4, 0, 0, 0, //
            0, 0, 0, 4, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 4, 0, //
            0, 4, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 4, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 4, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 4, 0, 0, //
            0, 0, 0, 4, 0, 0, 0, 0, 0, //
        ],
    )?;

    // Imagine this as commands to send to the game
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    enum Dir {
        #[default]
        None,
        Up,
        Down,
        Right,
        Left,
    }

    let mut a = Array2::<Dir>::from_elem((15, 9), Dir::default());

    Zip::from(&mut a)
        .and(&arr)
        .and(&arr2)
        .and(&arr3)
        .and(&arr4)
        .par_for_each(|a, &b, &c, &d, &f| {
            *a = if b > 0 {
                Dir::Up
            } else if c > 0 {
                Dir::Down
            } else if d > 0 {
                Dir::Right
            } else if f > 0 {
                Dir::Left
            } else {
                Dir::None
            };
        });

    #[derive(Debug, Clone, Default)]
    struct Bundle {
        pub dirs: Vec<Dir>,
        pub is_empty: bool,
    }

    let mut histogram: Vec<i32> = Vec::with_capacity(a.nrows());
    a.outer_iter()
        .into_par_iter()
        .map(|rows| {
            let sum = rows.fold(0, |acc, &dir| if dir != Dir::None { acc + 1 } else { acc });
            sum
        })
        .collect_into_vec(&mut histogram);

    let mut peaks: Vec<usize> = Vec::new();
    for (i, &el) in histogram.iter().enumerate() {
        let previous_bar = if i > 0 { histogram[i - 1] } else { 0 };
        let next_bar = if i < histogram.len() - 1 {
            histogram[i + 1]
        } else {
            0
        };
        if el > previous_bar && el > next_bar {
            peaks.push(i);
        }
    }

    println!("Elapsed: {:?}", now.elapsed());
    println!("Peaks: {:?}", peaks);

    // for window in a.axis_windows(Axis(0), 3) {
    //     println!("Window: {:?}", window);
    // }

    // println!("Source: {:?}", arr);
    println!("Res: {:?}", a);

    // for row in arr.axis_iter(Axis(0)) {
    //     println!("Row: {:?}", row);
    // }

    // // Traverse the array with a window size of 3 along axis 1
    // for (window, correct) in arr.axis_windows(Axis(0), 3).into_iter().zip(&correct) {
    //     println!("Window: {:?}", window);
    //     // assert_eq!(window, correct);
    //     // assert_eq!(window.shape(), &[4, 3, 2]);
    // }
    Ok(())
}
