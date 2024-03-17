use nd::parallel::prelude::IntoParallelIterator;
use ndarray::{self as nd, parallel::prelude::*};
use strum::Display;

#[derive(Clone, Copy, Debug, Default, Display, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Direction {
    #[default]
    None,
    Up,
    Right,
    Down,
    Left,
}

pub fn find_direction_commands(
    up: &nd::ArrayView2<f32>,
    right: &nd::ArrayView2<f32>,
    down: &nd::ArrayView2<f32>,
    left: &nd::ArrayView2<f32>,
    threshold: Option<f32>,
    search_chunk_size: Option<usize>,
) -> anyhow::Result<Vec<Vec<Direction>>> {
    let threshold = threshold.unwrap_or(1.0);
    let search_chunk_size = search_chunk_size.unwrap_or(3);
    let directions = raw_mats_to_direction_buffer(up, right, down, left, threshold)?;
    let commands = collect_direction_commands(&directions.view(), search_chunk_size)?;
    Ok(commands)
}

pub fn raw_mats_to_direction_buffer(
    up: &nd::ArrayView2<f32>,
    right: &nd::ArrayView2<f32>,
    down: &nd::ArrayView2<f32>,
    left: &nd::ArrayView2<f32>,
    threshold: f32,
) -> anyhow::Result<nd::Array2<Direction>> {
    if up.shape() != right.shape() || up.shape() != down.shape() || up.shape() != left.shape() {
        return Err(anyhow::anyhow!("All mats must have the same shape"));
    }
    let mut buf: nd::Array2<Direction> = nd::Array2::from_elem(up.dim(), Default::default());
    nd::Zip::from(&mut buf)
        .and(up)
        .and(right)
        .and(down)
        .and(left)
        .par_for_each(|buf, &up, &right, &down, &left| {
            let max = up.max(right).max(down).max(left);
            *buf = if max < threshold {
                Direction::None
            } else if max == up {
                Direction::Up
            } else if max == right {
                Direction::Right
            } else if max == down {
                Direction::Down
            } else {
                Direction::Left
            }
        });

    Ok(buf)
}

pub fn collect_direction_commands(
    buf: &nd::ArrayView2<Direction>,
    search_chunk_size: usize,
) -> anyhow::Result<Vec<Vec<Direction>>> {
    // Iterate over the windowed columns and collect the non-None directions.
    let chunks: Vec<Vec<Direction>> = buf
        .axis_windows(nd::Axis(0), search_chunk_size)
        .into_iter()
        .map(|rows| {
            /*
             * Here we have a chunk of the matrix, and we need to find the first non-None value in each column.
             *
             * Imagine we have this matrix for one of the windows:
             * ```
             * [0 0 0 0 0]
             * [1 0 1 0 0]
             * [0 1 1 1 0]
             * [0 1 1 0 0]
             * ```
             *
             * This will be iterated as:
             * ```
             * [0 1 0 0]
             * [0 0 1 1]
             * [0 1 1 1]
             * [0 0 1 0]
             * [0 0 0 0]
             * ```
             *
             * Notice each column is now a row, and we can find the first non-None value in each row.
             * In which you can think of a transposed version of the original matrix.
             *
             * Also, since we are running very large number of iterations, we need to parallelize this.
             */
            rows.axis_iter(nd::Axis(1))
                .into_par_iter()
                .map(|col| {
                    col.into_par_iter()
                        .find_first(|&&el| el != Direction::None)
                        .copied()
                        .unwrap_or(Direction::None)
                })
                .filter(|&dir| dir != Direction::None)
                .collect()
        })
        .collect();

    let histogram: Vec<usize> = chunks
        .clone()
        .into_par_iter()
        .map(|rows| rows.len())
        .collect();
    let mut peaks: Vec<usize> = Vec::new();
    for (i, &el) in histogram.iter().enumerate() {
        let previous_bar = if i > 0 { histogram[i - 1] } else { 0 };
        let next_bar = if i < histogram.len() - 1 {
            histogram[i + 1]
        } else {
            0
        };
        if el >= previous_bar && el > next_bar {
            peaks.push(i);
        }
    }

    let commands: Vec<Vec<Direction>> = peaks
        .iter()
        .filter_map(|&i| chunks.get(i))
        .cloned()
        .collect();

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_direction_commands() -> anyhow::Result<()> {
        let now = std::time::Instant::now();
        let arr = nd::Array2::<f32>::from_shape_vec(
            (15, 9),
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
            ],
        )?;
        let arr2 = nd::Array2::<f32>::from_shape_vec(
            (15, 9),
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
            ],
        )?;
        let arr3 = nd::Array2::<f32>::from_shape_vec(
            (15, 9),
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, //
                0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
            ],
        )?;
        let arr4 = nd::Array2::<f32>::from_shape_vec(
            (15, 9),
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
            ],
        )?;

        let buf = find_direction_commands(
            &arr.view(),
            &arr2.view(),
            &arr3.view(),
            &arr4.view(),
            None,
            None,
        )?;
        assert_eq!(
            buf,
            vec![
                vec![
                    Direction::Up,
                    Direction::Right,
                    Direction::Right,
                    Direction::Right
                ],
                vec![
                    Direction::Right,
                    Direction::Up,
                    Direction::Up,
                    Direction::Up,
                    Direction::Right,
                    Direction::Down
                ],
                vec![Direction::Left, Direction::Up]
            ]
        );

        println!("Elapsed: {:?}", now.elapsed());

        Ok(())
    }
}
