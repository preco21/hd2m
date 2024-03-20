use ndarray::{self as nd, parallel::prelude::*};

pub fn find_direction_commands(
    up: &nd::ArrayView2<f32>,
    right: &nd::ArrayView2<f32>,
    down: &nd::ArrayView2<f32>,
    left: &nd::ArrayView2<f32>,
    threshold: Option<f32>,
    search_chunk_size: Option<usize>,
) -> anyhow::Result<Vec<Vec<DirectionDescriptor>>> {
    let threshold = threshold.unwrap_or(0.9);
    let search_chunk_size = search_chunk_size.unwrap_or(3);
    let dir_buf = raw_mats_to_direction_buffer(up, right, down, left, threshold)?;
    let commands = collect_direction_commands(&dir_buf.view(), search_chunk_size)?;
    Ok(commands)
}

pub fn raw_mats_to_direction_buffer(
    up: &nd::ArrayView2<f32>,
    right: &nd::ArrayView2<f32>,
    down: &nd::ArrayView2<f32>,
    left: &nd::ArrayView2<f32>,
    threshold: f32,
) -> anyhow::Result<nd::Array2<IntermediaryDirection>> {
    if up.shape() != right.shape() || up.shape() != down.shape() || up.shape() != left.shape() {
        return Err(anyhow::anyhow!("All mats must have the same shape"));
    }

    let mut buf: nd::Array2<IntermediaryDirection> =
        nd::Array2::from_elem(up.dim(), Default::default());
    nd::Zip::from(&mut buf)
        .and(up)
        .and(right)
        .and(down)
        .and(left)
        .par_for_each(|buf, &up, &right, &down, &left| {
            let max = up.max(right).max(down).max(left);
            *buf = if max < threshold {
                None
            } else {
                let direction = if max == up {
                    Direction::Up
                } else if max == right {
                    Direction::Right
                } else if max == down {
                    Direction::Down
                } else {
                    Direction::Left
                };
                Some((direction, max))
            }
        });

    Ok(buf)
}

pub fn collect_direction_commands(
    buf: &nd::ArrayView2<IntermediaryDirection>,
    search_chunk_size: usize,
) -> anyhow::Result<Vec<Vec<DirectionDescriptor>>> {
    // Iterate over the windowed columns and collect the non-None directions.
    let chunks: Vec<Vec<DirectionDescriptor>> = buf
        .axis_windows(nd::Axis(0), search_chunk_size)
        .into_iter()
        .enumerate()
        .map(|(y, rows)| {
            /*
             * Here we have a chunk of the matrix, and we need to find the first non-None value in each column.
             *
             * Imagine we have this matrix for one of the windows:
             * ```
             * [0 0 0 0 0] |
             * [1 0 1 0 0] |
             * [0 1 1 1 0] |
             * [0 1 1 0 0] v
             * ```
             *
             * This will be iterated as:
             * ```
             * -------->
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
                .enumerate()
                .map(|(x, col)| {
                    col.into_par_iter()
                        .find_first(|&&el| el != None)
                        .and_then(|&dir| {
                            dir.and_then(|(direction, confidence)| {
                                Some(DirectionDescriptor {
                                    direction,
                                    position: Point { x, y },
                                    confidence,
                                })
                            })
                        })
                })
                .filter_map(|dir| dir)
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

    let descriptors: Vec<Vec<DirectionDescriptor>> = peaks
        .iter()
        .filter_map(|&i| chunks.get(i))
        .cloned()
        .collect();

    Ok(descriptors)
}

// Temporarily stores direction and f32 confidence.
pub type IntermediaryDirection = Option<(Direction, f32)>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionDescriptor {
    pub direction: Direction,
    pub position: Point,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
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
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 3, y: 2 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 4, y: 2 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 5, y: 2 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 6, y: 2 },
                        confidence: 1.0
                    }
                ],
                vec![
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 1, y: 6 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 2, y: 6 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 5, y: 6 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 6, y: 6 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 7, y: 6 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Down,
                        position: Point { x: 8, y: 6 },
                        confidence: 1.0
                    }
                ],
                vec![
                    DirectionDescriptor {
                        direction: Direction::Left,
                        position: Point { x: 3, y: 12 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 6, y: 12 },
                        confidence: 1.0
                    }
                ]
            ]
        );

        println!("Elapsed: {:?}", now.elapsed());

        Ok(())
    }
}
