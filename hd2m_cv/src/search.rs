use ndarray::{self as nd, parallel::prelude::*};
use std::collections::BTreeMap;

pub fn find_direction_commands(
    up: &nd::ArrayView2<f32>,
    down: &nd::ArrayView2<f32>,
    right: &nd::ArrayView2<f32>,
    left: &nd::ArrayView2<f32>,
    threshold: Option<f32>,
    search_chunk_size: Option<usize>,
    discarding_distance_threshold: Option<f64>,
) -> anyhow::Result<Vec<Vec<DirectionDescriptor>>> {
    let threshold = threshold.unwrap_or(0.9);
    let search_chunk_size = search_chunk_size.unwrap_or(3);
    let discarding_distance_threshold = discarding_distance_threshold.unwrap_or(30.0);
    let dir_buf = raw_mats_to_direction_buffer(up, right, down, left, threshold)?;
    let commands = collect_direction_commands(
        &dir_buf.view(),
        search_chunk_size,
        discarding_distance_threshold,
    )?;
    Ok(commands)
}

pub fn raw_mats_to_direction_buffer(
    up: &nd::ArrayView2<f32>,
    down: &nd::ArrayView2<f32>,
    right: &nd::ArrayView2<f32>,
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
        .par_for_each(|buf, &up, &down, &right, &left| {
            let max = up.max(down).max(right).max(left);
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
    discarding_distance_threshold: f64,
) -> anyhow::Result<Vec<Vec<DirectionDescriptor>>> {
    // Iterate over the windowed columns and collect the non-None directions.
    let chunks: Vec<Vec<DirectionDescriptor>> = buf
        .axis_windows(nd::Axis(1), search_chunk_size)
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>()
        .par_iter()
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
            let mut last_seen_points: BTreeMap<Direction, Point> = BTreeMap::new();
            rows.axis_iter(nd::Axis(0))
                .enumerate()
                .map(|(x, col)| {
                    col.iter().enumerate().find_map(|(k, &el)| {
                        let (direction, confidence) = el?;
                        Some(DirectionDescriptor {
                            direction,
                            position: Point::new(x, y + k),
                            confidence,
                        })
                    })
                })
                // Sanitize the too-close directions as well as `None` values.
                .filter_map(|dir| {
                    let desc = dir?;
                    let last_seen_point = last_seen_points
                        .get(&desc.direction)
                        .copied()
                        .unwrap_or(Default::default());
                    // Discard the direction if it's too close to the previous one.
                    if !desc.position.is_zero()
                        && last_seen_point.distance(desc.position) < discarding_distance_threshold
                    {
                        return None;
                    }
                    last_seen_points
                        .entry(desc.direction)
                        .and_modify(|e| *e = desc.position)
                        .or_insert(desc.position);
                    Some(desc)
                })
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

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn zeroed() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    pub fn distance(&self, other: Point) -> f64 {
        let x_diff = self.x as f64 - other.x as f64;
        let y_diff = self.y as f64 - other.y as f64;
        (x_diff.powi(2) + y_diff.powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_direction_commands() -> anyhow::Result<()> {
        let now = std::time::Instant::now();
        let up_arr = nd::Array2::<f32>::from_shape_vec(
            (15, 9), // x, y
            vec![
                // xV, y>
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                1.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 7.0, 0.0, 0.0, //
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, //
            ],
        )?;
        let down_arr = nd::Array2::<f32>::from_shape_vec(
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
        let right_arr = nd::Array2::<f32>::from_shape_vec(
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
        let left_arr = nd::Array2::<f32>::from_shape_vec(
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
            &up_arr.view(),
            &down_arr.view(),
            &right_arr.view(),
            &left_arr.view(),
            None,
            None,
            Some(2.0),
        )?;
        assert_eq!(
            buf,
            vec![
                vec![
                    DirectionDescriptor {
                        direction: Direction::Left,
                        position: Point { x: 1, y: 3 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 4, y: 3 },
                        confidence: 2.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Left,
                        position: Point { x: 5, y: 3 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 7, y: 1 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 8, y: 2 },
                        confidence: 5.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Left,
                        position: Point { x: 14, y: 3 },
                        confidence: 1.0
                    }
                ],
                vec![
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 2, y: 6 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Right,
                        position: Point { x: 4, y: 5 },
                        confidence: 1.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 6, y: 6 },
                        confidence: 3.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 10, y: 6 },
                        confidence: 6.0
                    },
                    DirectionDescriptor {
                        direction: Direction::Up,
                        position: Point { x: 13, y: 6 },
                        confidence: 7.0
                    }
                ]
            ]
        );

        println!("Elapsed: {:?}", now.elapsed());

        Ok(())
    }
}
